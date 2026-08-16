use super::super::local_store::{self, LocalStore};
use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::token_session::{token_session, TokenLease};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteTokenState {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub access_token_expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteTokenSyncStatus {
    pub has_access_token: bool,
    pub has_refresh_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteSearchResult<T> {
    // Daylite omits `results` entirely (returning a bare `{}`) when a search has
    // no matches, so default to an empty list instead of failing to deserialize.
    #[serde(default = "Vec::new")]
    pub results: Vec<T>,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteApiError {
    pub code: DayliteApiErrorCode,
    pub http_status: Option<u16>,
    pub user_message: String,
    pub technical_message: String,
}

impl DayliteApiError {
    pub(super) fn new(
        code: DayliteApiErrorCode,
        http_status: Option<u16>,
        user_message: impl Into<String>,
        technical_message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            http_status,
            user_message: user_message.into(),
            technical_message: technical_message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayliteApiErrorCode {
    Unauthorized,
    RateLimited,
    ServerError,
    MissingToken,
    InvalidConfiguration,
    RequestFailed,
    InvalidResponse,
    TokenRefreshFailed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteRefreshTokenRequest {
    pub base_url: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DayliteSearchSort {
    #[default]
    Id,
    Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteSearchInput {
    pub search_term: String,
    pub limit: Option<u16>,
    #[serde(default)]
    pub statuses: Option<Vec<String>>,
    #[serde(default)]
    pub full_records: Option<bool>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub sort: Option<DayliteSearchSort>,
}

pub(super) fn build_limit_query(limit: Option<u16>) -> Vec<(String, String)> {
    let mut query = Vec::new();
    if let Some(limit) = limit {
        query.push(("limit".to_string(), limit.to_string()));
    }

    query
}

pub(super) fn normalize_base_url(base_url: &str) -> Result<String, DayliteApiError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(DayliteApiError::new(
            DayliteApiErrorCode::InvalidConfiguration,
            None,
            "Die Daylite-URL ist nicht konfiguriert.",
            "Leere dayliteBaseUrl-Konfiguration",
        ));
    }

    Ok(trimmed.to_string())
}

pub(super) fn load_daylite_tokens() -> Result<DayliteTokenState, DayliteApiError> {
    match crate::secret_manager::get_token("lkr-planner-daylite", "LKR Planner Daylite Token") {
        Ok(json_str) => serde_json::from_str(&json_str).map_err(|e| {
            DayliteApiError::new(
                DayliteApiErrorCode::InvalidConfiguration,
                None,
                "Die gespeicherten Daylite-Zugangsdaten sind beschädigt. Bitte verbinde dich erneut.",
                format!("Token-JSON konnte nicht deserialisiert werden: {e}"),
            )
        }),
        Err(crate::secret_manager::SecretError::NotFound) => Ok(DayliteTokenState::default()),
        Err(e) => Err(DayliteApiError::new(
            DayliteApiErrorCode::InvalidConfiguration,
            None,
            "Auf die Daylite-Zugangsdaten im Keychain konnte nicht zugegriffen werden. Bitte prüfe die Keychain-Berechtigungen.",
            format!("Keychain-Fehler beim Lesen des Daylite-Tokens: {e}"),
        )),
    }
}

pub(super) fn store_daylite_tokens(token_state: &DayliteTokenState) -> Result<(), DayliteApiError> {
    let json_str = serde_json::to_string(token_state).map_err(|e| {
        DayliteApiError::new(
            DayliteApiErrorCode::ServerError,
            None,
            "Token konnten nicht sicher gespeichert werden.",
            format!("Token serialization failed: {e}"),
        )
    })?;

    crate::secret_manager::set_token(
        "lkr-planner-daylite",
        "LKR Planner Daylite Token",
        &json_str,
    )
    .map_err(|e| {
        DayliteApiError::new(
            DayliteApiErrorCode::ServerError,
            None,
            "Auf den sicheren Speicher konnte nicht zugegriffen werden (Keychain verweigert?).",
            e.to_string(),
        )
    })
}

async fn lease_tokens(client: &DayliteApiClient) -> Result<TokenLease, DayliteApiError> {
    let session = token_session();
    if session.is_empty() {
        session.seed(load_daylite_tokens()?);
    }

    session
        .lease(current_epoch_ms()?, |current| {
            rotate_tokens(client, current)
        })
        .await
}

async fn rotate_tokens(
    client: &DayliteApiClient,
    current: DayliteTokenState,
) -> Result<DayliteTokenState, DayliteApiError> {
    if current.access_token.trim().is_empty() && current.refresh_token.trim().is_empty() {
        return Err(missing_token_error(
            "Es sind keine Daylite-Zugangsdaten hinterlegt. Bitte ein Refresh-Token hinterlegen.",
            "Weder Access- noch Refresh-Token sind vorhanden.",
        ));
    }

    let rotated = refresh_tokens(client, current.refresh_token).await?;
    store_daylite_tokens(&rotated)?;

    Ok(rotated)
}

/// Retries once on rejection: a token revoked server-side defeats expiry-driven rotation.
pub(super) async fn with_daylite_tokens<T, F, Fut>(
    client: &DayliteApiClient,
    operation: F,
) -> Result<T, DayliteApiError>
where
    F: Fn(DayliteTokenState) -> Fut,
    Fut: std::future::Future<Output = Result<T, DayliteApiError>>,
{
    let lease = lease_tokens(client).await?;
    retry_once_on_unauthorized(lease.tokens.clone(), operation, || async {
        let renewed = token_session()
            .renew(&lease, |current| rotate_tokens(client, current))
            .await?;
        Ok(renewed.tokens)
    })
    .await
}

async fn retry_once_on_unauthorized<T, F, Fut, R, RFut>(
    tokens: DayliteTokenState,
    operation: F,
    renew: R,
) -> Result<T, DayliteApiError>
where
    F: Fn(DayliteTokenState) -> Fut,
    Fut: std::future::Future<Output = Result<T, DayliteApiError>>,
    R: FnOnce() -> RFut,
    RFut: std::future::Future<Output = Result<DayliteTokenState, DayliteApiError>>,
{
    match operation(tokens).await {
        Err(error) if error.code == DayliteApiErrorCode::Unauthorized => {
            operation(renew().await?).await
        }
        result => result,
    }
}

/// For operations that cannot be replayed, such as one that has already mutated the store.
pub(super) async fn with_daylite_tokens_once<Fut>(
    client: &DayliteApiClient,
    operation: impl FnOnce(DayliteTokenState) -> Fut,
) -> Result<(), DayliteApiError>
where
    Fut: std::future::Future<Output = Result<(), DayliteApiError>>,
{
    let lease = lease_tokens(client).await?;
    operation(lease.tokens).await
}

/// For read-only command bodies only: commands that mutate the local store manage the store themselves.
pub(super) async fn run_daylite_command<T>(
    app: tauri::AppHandle,
    operation: impl AsyncFn(&DayliteApiClient, DayliteTokenState) -> Result<T, DayliteApiError>,
) -> Result<T, DayliteApiError> {
    let store = load_store_or_error(app)?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    with_daylite_tokens(&client, |tokens| operation(&client, tokens)).await
}

pub(super) async fn adopt_daylite_tokens<F, Fut>(
    mint: F,
) -> Result<DayliteTokenState, DayliteApiError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<DayliteTokenState, DayliteApiError>>,
{
    let lease = token_session()
        .adopt(|| async {
            let minted = mint().await?;
            store_daylite_tokens(&minted)?;
            Ok(minted)
        })
        .await?;

    Ok(lease.tokens)
}

pub(super) fn load_store_or_error(app: tauri::AppHandle) -> Result<LocalStore, DayliteApiError> {
    local_store::load_local_store(app).map_err(map_store_error)
}

pub(super) fn save_store_or_error(
    app: tauri::AppHandle,
    store: LocalStore,
) -> Result<(), DayliteApiError> {
    local_store::save_local_store(app, store).map_err(map_store_error)
}

pub(super) fn normalize_http_error(status: u16, body: &str, path: &str) -> DayliteApiError {
    let (code, user_message) = if status == 401 {
        (
            DayliteApiErrorCode::Unauthorized,
            "Die Daylite-Anmeldung ist abgelaufen oder ungültig.",
        )
    } else if status == 429 {
        (
            DayliteApiErrorCode::RateLimited,
            "Daylite hat zu viele Anfragen erhalten. Bitte kurz warten und erneut versuchen.",
        )
    } else if (500..=599).contains(&status) {
        (
            DayliteApiErrorCode::ServerError,
            "Daylite ist aktuell nicht erreichbar.",
        )
    } else {
        (
            DayliteApiErrorCode::RequestFailed,
            "Die Daten konnten nicht von Daylite geladen werden.",
        )
    };

    DayliteApiError::new(
        code,
        Some(status),
        user_message,
        format!(
            "Daylite request failed for {path} with status={status}; body={}",
            truncate_for_log(body)
        ),
    )
}

pub(super) fn parse_success_json_body<T: DeserializeOwned>(
    status: u16,
    body: &str,
    path: &str,
) -> Result<T, DayliteApiError> {
    if !(200..300).contains(&status) {
        return Err(normalize_http_error(status, body, path));
    }

    parse_json_body(status, body, path)
}

pub(super) fn parse_json_body<T: DeserializeOwned>(
    status: u16,
    body: &str,
    path: &str,
) -> Result<T, DayliteApiError> {
    serde_json::from_str::<T>(body).map_err(|error| {
        DayliteApiError::new(
            DayliteApiErrorCode::InvalidResponse,
            Some(status),
            "Die Antwort von Daylite konnte nicht verarbeitet werden.",
            format!(
                "JSON-Verarbeitung für {path} fehlgeschlagen: {error}; body={}",
                truncate_for_log(body)
            ),
        )
    })
}

pub(super) fn missing_token_error(user_message: &str, technical_message: &str) -> DayliteApiError {
    DayliteApiError::new(
        DayliteApiErrorCode::MissingToken,
        None,
        user_message,
        technical_message,
    )
}

pub(super) fn current_epoch_ms() -> Result<u64, DayliteApiError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            DayliteApiError::new(
                DayliteApiErrorCode::RequestFailed,
                None,
                "Die aktuelle Systemzeit konnte nicht gelesen werden.",
                format!("Systemzeitfehler: {error}"),
            )
        })?;

    u64::try_from(duration.as_millis()).map_err(|error| {
        DayliteApiError::new(
            DayliteApiErrorCode::RequestFailed,
            None,
            "Die aktuelle Systemzeit konnte nicht gelesen werden.",
            format!("Zeitstempel-Konvertierung fehlgeschlagen: {error}"),
        )
    })
}

/// Daylite returns padded strings and uses "" where it means "absent", so every
/// mapped field is trimmed and an empty result collapses to None.
pub(super) fn trimmed_or_none(value: Option<String>) -> Option<String> {
    let trimmed = value?.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(super) fn trimmed(value: String) -> String {
    value.trim().to_string()
}

pub(super) fn truncate_for_log(value: &str) -> String {
    let limit = 400;
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn map_store_error(error: local_store::StoreError) -> DayliteApiError {
    DayliteApiError::new(
        DayliteApiErrorCode::InvalidConfiguration,
        None,
        error.user_message,
        error.technical_message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn tokens(access: &str) -> DayliteTokenState {
        DayliteTokenState {
            access_token: access.to_string(),
            refresh_token: "rt".to_string(),
            access_token_expires_at_ms: Some(900_000),
        }
    }

    fn unauthorized() -> DayliteApiError {
        DayliteApiError::new(
            DayliteApiErrorCode::Unauthorized,
            Some(401),
            "abgelaufen",
            "401",
        )
    }

    #[tokio::test]
    async fn a_successful_call_does_not_renew() {
        let renewals = AtomicUsize::new(0);

        let value = retry_once_on_unauthorized(
            tokens("at"),
            |_| async { Ok("ok") },
            || async {
                renewals.fetch_add(1, Ordering::SeqCst);
                Ok(tokens("renewed"))
            },
        )
        .await
        .expect("call should succeed");

        assert_eq!(value, "ok");
        assert_eq!(renewals.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn a_rejected_token_is_renewed_and_the_call_replayed() {
        let attempts = AtomicUsize::new(0);
        let seen = std::sync::Mutex::new(Vec::new());

        let value = retry_once_on_unauthorized(
            tokens("at"),
            |current| {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                seen.lock().unwrap().push(current.access_token.clone());
                async move {
                    if attempt == 0 {
                        return Err(unauthorized());
                    }
                    Ok("ok")
                }
            },
            || async { Ok(tokens("renewed")) },
        )
        .await
        .expect("call should succeed after renewal");

        assert_eq!(value, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(*seen.lock().unwrap(), vec!["at", "renewed"]);
    }

    #[tokio::test]
    async fn a_second_rejection_is_not_retried_again() {
        let attempts = AtomicUsize::new(0);

        let error = retry_once_on_unauthorized(
            tokens("at"),
            |_| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async { Err::<&str, _>(unauthorized()) }
            },
            || async { Ok(tokens("renewed")) },
        )
        .await
        .expect_err("a second rejection should surface");

        assert_eq!(error.code, DayliteApiErrorCode::Unauthorized);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn an_error_that_is_not_a_rejection_does_not_renew() {
        let renewals = AtomicUsize::new(0);

        let error = retry_once_on_unauthorized(
            tokens("at"),
            |_| async {
                Err::<&str, _>(DayliteApiError::new(
                    DayliteApiErrorCode::RateLimited,
                    Some(429),
                    "zu viele",
                    "429",
                ))
            },
            || async {
                renewals.fetch_add(1, Ordering::SeqCst);
                Ok(tokens("renewed"))
            },
        )
        .await
        .expect_err("the error should surface");

        assert_eq!(error.code, DayliteApiErrorCode::RateLimited);
        assert_eq!(renewals.load(Ordering::SeqCst), 0);
    }
}
