use super::super::local_store::{self, LocalStore};
use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::token_session::{token_session, TokenLease};
use crate::error::{AppError, AppErrorCode};
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
    // Daylite returns a bare `{}` without `results` when nothing matches.
    #[serde(default = "Vec::new")]
    pub results: Vec<T>,
    #[serde(default)]
    pub next: Option<String>,
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

pub(super) fn normalize_base_url(base_url: &str) -> Result<String, AppError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AppError::new(
            AppErrorCode::InvalidConfiguration,
            "Die Daylite-URL ist nicht konfiguriert.",
            "Leere dayliteBaseUrl-Konfiguration",
        ));
    }

    Ok(trimmed.to_string())
}

pub(super) fn load_daylite_tokens() -> Result<DayliteTokenState, AppError> {
    match crate::secret_manager::get_token("lkr-planner-daylite", "LKR Planner Daylite Token") {
        Ok(json_str) => serde_json::from_str(&json_str).map_err(|e| {
            AppError::new(AppErrorCode::InvalidConfiguration, "Die gespeicherten Daylite-Zugangsdaten sind beschädigt. Bitte verbinde dich erneut.", format!("Token-JSON konnte nicht deserialisiert werden: {e}"))
        }),
        Err(crate::secret_manager::SecretError::NotFound) => Ok(DayliteTokenState::default()),
        Err(e) => Err(AppError::new(AppErrorCode::InvalidConfiguration, "Auf die Daylite-Zugangsdaten im Keychain konnte nicht zugegriffen werden. Bitte prüfe die Keychain-Berechtigungen.", format!("Keychain-Fehler beim Lesen des Daylite-Tokens: {e}"))),
    }
}

pub(super) fn store_daylite_tokens(token_state: &DayliteTokenState) -> Result<(), AppError> {
    let json_str = serde_json::to_string(token_state).map_err(|e| {
        AppError::new(
            AppErrorCode::ServerError,
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
        AppError::new(
            AppErrorCode::ServerError,
            "Auf den sicheren Speicher konnte nicht zugegriffen werden (Keychain verweigert?).",
            e.to_string(),
        )
    })
}

async fn lease_tokens(client: &DayliteApiClient) -> Result<TokenLease, AppError> {
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
) -> Result<DayliteTokenState, AppError> {
    if current.access_token.trim().is_empty() && current.refresh_token.trim().is_empty() {
        return Err(missing_token_error(
            "Es sind keine Daylite-Zugangsdaten hinterlegt. Bitte ein Refresh-Token hinterlegen.",
            "Weder Access- noch Refresh-Token sind vorhanden.",
        ));
    }

    refresh_tokens(client, current.refresh_token).await
}

/// Retries once on rejection: a token revoked server-side defeats expiry-driven rotation.
pub(super) async fn with_daylite_tokens<T, F, Fut>(
    client: &DayliteApiClient,
    operation: F,
) -> Result<T, AppError>
where
    F: Fn(DayliteTokenState) -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
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
) -> Result<T, AppError>
where
    F: Fn(DayliteTokenState) -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
    R: FnOnce() -> RFut,
    RFut: std::future::Future<Output = Result<DayliteTokenState, AppError>>,
{
    match operation(tokens).await {
        Err(error) if error.code == AppErrorCode::Unauthorized => operation(renew().await?).await,
        result => result,
    }
}

/// For operations that cannot be replayed, such as one that has already mutated the store.
pub(super) async fn with_daylite_tokens_once<Fut>(
    client: &DayliteApiClient,
    operation: impl FnOnce(DayliteTokenState) -> Fut,
) -> Result<(), AppError>
where
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let lease = lease_tokens(client).await?;
    operation(lease.tokens).await
}

/// Only for read-only bodies; mutating commands manage the local store themselves.
pub(super) async fn run_daylite_command<T>(
    app: tauri::AppHandle,
    operation: impl AsyncFn(&DayliteApiClient, DayliteTokenState) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?.with_telemetry(&app);
    with_daylite_tokens(&client, |tokens| operation(&client, tokens)).await
}

pub(super) async fn adopt_daylite_tokens<F, Fut>(mint: F) -> Result<DayliteTokenState, AppError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<DayliteTokenState, AppError>>,
{
    token_session().adopt(mint).await
}

/// Must run before the process exits, including the restart that installs an update.
pub async fn close_daylite_token_session() -> Result<(), AppError> {
    token_session().close().await
}

pub(super) fn load_store_or_error(app: tauri::AppHandle) -> Result<LocalStore, AppError> {
    local_store::load_local_store(app).map_err(map_store_error)
}

pub(super) fn save_store_or_error(
    app: tauri::AppHandle,
    store: LocalStore,
) -> Result<(), AppError> {
    local_store::save_local_store(app, store).map_err(map_store_error)
}

pub(super) fn normalize_http_error(status: u16, body: &str, path: &str) -> AppError {
    let (code, user_message) = if status == 401 {
        (
            AppErrorCode::Unauthorized,
            "Die Daylite-Anmeldung ist abgelaufen oder ungültig.",
        )
    } else if status == 429 {
        (
            AppErrorCode::RateLimited,
            "Daylite hat zu viele Anfragen erhalten. Bitte kurz warten und erneut versuchen.",
        )
    } else if (500..=599).contains(&status) {
        (
            AppErrorCode::ServerError,
            "Daylite ist aktuell nicht erreichbar.",
        )
    } else {
        (
            AppErrorCode::RequestFailed,
            "Die Daten konnten nicht von Daylite geladen werden.",
        )
    };

    AppError::new(
        code,
        user_message,
        format!(
            "Daylite request failed for {path} with status={status}; body={}",
            truncate_for_log(body)
        ),
    )
    .with_http_status(Some(status))
}

pub(super) fn parse_success_json_body<T: DeserializeOwned>(
    status: u16,
    body: &str,
    path: &str,
) -> Result<T, AppError> {
    if !(200..300).contains(&status) {
        return Err(normalize_http_error(status, body, path));
    }

    parse_json_body(status, body, path)
}

pub(super) fn parse_json_body<T: DeserializeOwned>(
    status: u16,
    body: &str,
    path: &str,
) -> Result<T, AppError> {
    serde_json::from_str::<T>(body).map_err(|error| {
        AppError::new(
            AppErrorCode::InvalidResponse,
            "Die Antwort von Daylite konnte nicht verarbeitet werden.",
            format!(
                "JSON-Verarbeitung für {path} fehlgeschlagen: {error}; body={}",
                truncate_for_log(body)
            ),
        )
        .with_http_status(Some(status))
    })
}

pub(super) fn missing_token_error(user_message: &str, technical_message: &str) -> AppError {
    AppError::new(AppErrorCode::MissingToken, user_message, technical_message)
}

pub(super) fn current_epoch_ms() -> Result<u64, AppError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AppError::new(
                AppErrorCode::RequestFailed,
                "Die aktuelle Systemzeit konnte nicht gelesen werden.",
                format!("Systemzeitfehler: {error}"),
            )
        })?;

    u64::try_from(duration.as_millis()).map_err(|error| {
        AppError::new(
            AppErrorCode::RequestFailed,
            "Die aktuelle Systemzeit konnte nicht gelesen werden.",
            format!("Zeitstempel-Konvertierung fehlgeschlagen: {error}"),
        )
    })
}

/// Daylite pads strings and encodes absent values as "".
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

fn map_store_error(error: AppError) -> AppError {
    AppError::new(
        AppErrorCode::InvalidConfiguration,
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

    fn unauthorized() -> AppError {
        AppError::new(AppErrorCode::Unauthorized, "abgelaufen", "401").with_http_status(Some(401))
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

        assert_eq!(error.code, AppErrorCode::Unauthorized);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn an_error_that_is_not_a_rejection_does_not_renew() {
        let renewals = AtomicUsize::new(0);

        let error = retry_once_on_unauthorized(
            tokens("at"),
            |_| async {
                Err::<&str, _>(
                    AppError::new(AppErrorCode::RateLimited, "zu viele", "429")
                        .with_http_status(Some(429)),
                )
            },
            || async {
                renewals.fetch_add(1, Ordering::SeqCst);
                Ok(tokens("renewed"))
            },
        )
        .await
        .expect_err("the error should surface");

        assert_eq!(error.code, AppErrorCode::RateLimited);
        assert_eq!(renewals.load(Ordering::SeqCst), 0);
    }
}
