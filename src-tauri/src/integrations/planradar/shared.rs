use super::super::local_store::{self, LocalStore};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Keychain coordinates for the Planradar API token, mirroring the Daylite convention
/// (`lkr-planner-daylite` / `LKR Planner Daylite Token`).
pub(super) const PLANRADAR_KEYCHAIN_SERVICE: &str = "lkr-planner-planradar";
pub(super) const PLANRADAR_KEYCHAIN_USERNAME: &str = "LKR Planner Planradar Token";

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarApiError {
    pub code: PlanradarApiErrorCode,
    pub http_status: Option<u16>,
    pub user_message: String,
    pub technical_message: String,
}

impl PlanradarApiError {
    pub(super) fn new(
        code: PlanradarApiErrorCode,
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
pub enum PlanradarApiErrorCode {
    Unauthorized,
    RateLimited,
    ServerError,
    MissingToken,
    MissingCustomerId,
    InvalidConfiguration,
    RequestFailed,
    InvalidResponse,
    NotFound,
    Timeout,
}

/// Resolved, non-secret Planradar configuration (base URL plus Customer ID path segment).
/// The API token is loaded separately from the keychain via [`load_api_token`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlanradarConfig {
    pub base_url: String,
    pub customer_id: String,
}

/// Normalizes the configured Planradar base URL to the scheme+host root the client expects.
/// Request paths already carry the `/api/v1/...` prefix, so a base URL that the user pasted
/// with a trailing `/api` (or `/`) is trimmed here to avoid building a doubled `/api/api/v1/...`.
pub(super) fn normalize_base_url(base_url: &str) -> Result<String, PlanradarApiError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let trimmed = trimmed.strip_suffix("/api").unwrap_or(trimmed);
    if trimmed.is_empty() {
        return Err(PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidConfiguration,
            None,
            "Die Planradar-URL ist nicht konfiguriert.",
            "Leere planradarBaseUrl-Konfiguration",
        ));
    }

    Ok(trimmed.to_string())
}

/// Validates a value that is interpolated into a request path (Customer ID, project ID).
///
/// Path segments are concatenated into the URL before parsing, and `Url::parse` resolves dot
/// segments, so an unchecked value such as `../../9999/projects/5` would escape the configured
/// customer scope while still carrying the account token. Planradar identifiers are numeric (or
/// at most alphanumeric with dashes), so anything outside that set is rejected outright rather
/// than escaped.
pub(super) fn validate_path_segment(
    value: &str,
    label: &str,
    user_message: &str,
) -> Result<String, PlanradarApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.chars().all(is_safe_path_segment_char) {
        return Err(PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidConfiguration,
            None,
            user_message,
            format!("Ungültiger Wert für {label}: {trimmed:?}"),
        ));
    }

    Ok(trimmed.to_string())
}

fn is_safe_path_segment_char(candidate: char) -> bool {
    candidate.is_ascii_alphanumeric() || candidate == '-' || candidate == '_'
}

/// Reads the configured Planradar base URL and Customer ID from the local store, validating
/// that both are present so callers fail fast with a German message instead of building an
/// invalid request path.
pub(super) fn load_config(store: &LocalStore) -> Result<PlanradarConfig, PlanradarApiError> {
    let base_url = normalize_base_url(&store.api_endpoints.planradar_base_url)?;
    let customer_id = store.api_endpoints.planradar_customer_id.trim().to_string();
    if customer_id.is_empty() {
        return Err(PlanradarApiError::new(
            PlanradarApiErrorCode::MissingCustomerId,
            None,
            "Die Planradar Customer ID ist nicht konfiguriert. Bitte in den Einstellungen hinterlegen.",
            "Leere planradarCustomerId-Konfiguration",
        ));
    }

    let customer_id = validate_path_segment(
        &customer_id,
        "planradarCustomerId",
        "Die Planradar Customer ID ist ungültig. Bitte in den Einstellungen korrigieren.",
    )?;

    Ok(PlanradarConfig {
        base_url,
        customer_id,
    })
}

pub(super) fn load_api_token() -> Result<String, PlanradarApiError> {
    match crate::secret_manager::get_token(PLANRADAR_KEYCHAIN_SERVICE, PLANRADAR_KEYCHAIN_USERNAME) {
        Ok(token) if token.trim().is_empty() => Err(missing_token_error()),
        Ok(token) => Ok(token),
        Err(crate::secret_manager::SecretError::NotFound) => Err(missing_token_error()),
        Err(e) => Err(PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidConfiguration,
            None,
            "Auf das Planradar-Token im Keychain konnte nicht zugegriffen werden. Bitte prüfe die Keychain-Berechtigungen.",
            format!("Keychain-Fehler beim Lesen des Planradar-Tokens: {e}"),
        )),
    }
}

pub(super) fn store_api_token(token: &str) -> Result<(), PlanradarApiError> {
    crate::secret_manager::set_token(
        PLANRADAR_KEYCHAIN_SERVICE,
        PLANRADAR_KEYCHAIN_USERNAME,
        token,
    )
    .map_err(|e| {
        PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidConfiguration,
            None,
            "Das Planradar-Token konnte nicht sicher gespeichert werden (Keychain verweigert?).",
            e.to_string(),
        )
    })
}

/// Snapshot of the token stored before a `planradar_connect` overwrites it, used to roll back if
/// the connect later fails. A read error is kept distinct from "definitely nothing stored": only
/// the latter may trigger a destructive rollback, because deleting on an unreadable keychain
/// would wipe a token that was actually still there.
pub(super) enum PreviousToken {
    Present(String),
    Absent,
    Unknown,
}

/// Returns the currently stored token, if any, without treating "not set" as an error. Used by
/// `planradar_connect` to snapshot the previous token so it can be restored if the connect fails
/// after the new token was written.
pub(super) fn peek_api_token() -> PreviousToken {
    match crate::secret_manager::get_token(PLANRADAR_KEYCHAIN_SERVICE, PLANRADAR_KEYCHAIN_USERNAME)
    {
        Ok(token) if !token.trim().is_empty() => PreviousToken::Present(token),
        Ok(_) => PreviousToken::Absent,
        Err(crate::secret_manager::SecretError::NotFound) => PreviousToken::Absent,
        // A transient read failure (e.g. AccessDenied): we cannot tell whether a token exists,
        // so the caller must not delete anything.
        Err(_) => PreviousToken::Unknown,
    }
}

/// Removes the stored Planradar token. Used to roll back a partial `planradar_connect` so a
/// token is never left in the keychain without its matching config in the store.
pub(super) fn delete_api_token() -> Result<(), PlanradarApiError> {
    match crate::secret_manager::delete_token(
        PLANRADAR_KEYCHAIN_SERVICE,
        PLANRADAR_KEYCHAIN_USERNAME,
    ) {
        Ok(()) | Err(crate::secret_manager::SecretError::NotFound) => Ok(()),
        Err(e) => Err(PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidConfiguration,
            None,
            "Das Planradar-Token konnte nicht aus dem Keychain entfernt werden.",
            e.to_string(),
        )),
    }
}

fn missing_token_error() -> PlanradarApiError {
    PlanradarApiError::new(
        PlanradarApiErrorCode::MissingToken,
        None,
        "Es ist kein Planradar-Token hinterlegt. Bitte ein gültiges API-Token hinterlegen.",
        "Kein Planradar-Token im Keychain vorhanden.",
    )
}

pub(super) fn load_store_or_error(app: tauri::AppHandle) -> Result<LocalStore, PlanradarApiError> {
    local_store::load_local_store(app).map_err(map_store_error)
}

pub(super) fn save_store_or_error(
    app: tauri::AppHandle,
    store: LocalStore,
) -> Result<(), PlanradarApiError> {
    local_store::save_local_store(app, store).map_err(map_store_error)
}

pub(super) fn normalize_http_error(status: u16, body: &str, path: &str) -> PlanradarApiError {
    let (code, user_message) = if status == 401 || status == 403 {
        (
            PlanradarApiErrorCode::Unauthorized,
            "Das Planradar-Token ist ungültig oder hat keine Berechtigung.",
        )
    } else if status == 404 {
        (
            PlanradarApiErrorCode::NotFound,
            "Das angeforderte Planradar-Projekt wurde nicht gefunden.",
        )
    } else if status == 429 {
        (
            PlanradarApiErrorCode::RateLimited,
            "Planradar hat zu viele Anfragen erhalten. Bitte kurz warten und erneut versuchen.",
        )
    } else if (500..=599).contains(&status) {
        (
            PlanradarApiErrorCode::ServerError,
            "Planradar ist aktuell nicht erreichbar.",
        )
    } else {
        (
            PlanradarApiErrorCode::RequestFailed,
            "Die Anfrage an Planradar ist fehlgeschlagen.",
        )
    };

    PlanradarApiError::new(
        code,
        Some(status),
        user_message,
        format!(
            "Planradar request failed for {path} with status={status}; body={}",
            truncate_for_log(body)
        ),
    )
}

pub(super) fn parse_success_json_body<T: DeserializeOwned>(
    status: u16,
    body: &str,
    path: &str,
) -> Result<T, PlanradarApiError> {
    if !(200..300).contains(&status) {
        return Err(normalize_http_error(status, body, path));
    }

    serde_json::from_str::<T>(body).map_err(|error| {
        PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidResponse,
            Some(status),
            "Die Antwort von Planradar konnte nicht verarbeitet werden.",
            format!(
                "JSON-Verarbeitung für {path} fehlgeschlagen: {error}; body={}",
                truncate_for_log(body)
            ),
        )
    })
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

fn map_store_error(error: local_store::StoreError) -> PlanradarApiError {
    PlanradarApiError::new(
        PlanradarApiErrorCode::InvalidConfiguration,
        None,
        error.user_message,
        error.technical_message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_base_url_trims_trailing_slash() {
        let normalized =
            normalize_base_url("https://www.planradar.com/").expect("should normalize");
        assert_eq!(normalized, "https://www.planradar.com");
    }

    #[test]
    fn normalize_base_url_strips_trailing_api_segment() {
        // Request paths already carry `/api/v1/...`, so a base URL pasted with `/api` must be
        // trimmed to avoid a doubled `/api/api/v1/...` path.
        assert_eq!(
            normalize_base_url("https://www.planradar.com/api").expect("should normalize"),
            "https://www.planradar.com"
        );
        assert_eq!(
            normalize_base_url("https://www.planradar.com/api/").expect("should normalize"),
            "https://www.planradar.com"
        );
        // A host that merely ends in "api" (no path segment) must be left intact.
        assert_eq!(
            normalize_base_url("https://myapi.example.com").expect("should normalize"),
            "https://myapi.example.com"
        );
    }

    #[test]
    fn normalize_base_url_rejects_blank() {
        let error = normalize_base_url("   ").expect_err("blank should fail");
        assert_eq!(error.code, PlanradarApiErrorCode::InvalidConfiguration);
    }

    #[test]
    fn load_config_requires_customer_id() {
        let mut store = LocalStore::default();
        store.api_endpoints.planradar_base_url = "https://www.planradar.com".to_string();
        store.api_endpoints.planradar_customer_id = "  ".to_string();

        let error = load_config(&store).expect_err("missing customer id should fail");
        assert_eq!(error.code, PlanradarApiErrorCode::MissingCustomerId);
    }

    #[test]
    fn load_config_returns_base_url_and_customer_id() {
        let mut store = LocalStore::default();
        store.api_endpoints.planradar_base_url = "https://www.planradar.com/".to_string();
        store.api_endpoints.planradar_customer_id = "4242".to_string();

        let config = load_config(&store).expect("config should resolve");
        assert_eq!(config.base_url, "https://www.planradar.com");
        assert_eq!(config.customer_id, "4242");
    }

    #[test]
    fn normalize_http_error_maps_status_codes() {
        assert_eq!(
            normalize_http_error(401, "", "/x").code,
            PlanradarApiErrorCode::Unauthorized
        );
        assert_eq!(
            normalize_http_error(403, "", "/x").code,
            PlanradarApiErrorCode::Unauthorized
        );
        assert_eq!(
            normalize_http_error(404, "", "/x").code,
            PlanradarApiErrorCode::NotFound
        );
        assert_eq!(
            normalize_http_error(429, "", "/x").code,
            PlanradarApiErrorCode::RateLimited
        );
        assert_eq!(
            normalize_http_error(503, "", "/x").code,
            PlanradarApiErrorCode::ServerError
        );
        assert_eq!(
            normalize_http_error(418, "", "/x").code,
            PlanradarApiErrorCode::RequestFailed
        );
    }
}
