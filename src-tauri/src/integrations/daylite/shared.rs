use super::super::local_store::{self, LocalStore};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub results: Vec<T>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteRefreshTokenRequest {
    pub base_url: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteSearchInput {
    pub search_term: String,
    pub limit: Option<u16>,
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
        return Err(DayliteApiError {
            code: DayliteApiErrorCode::InvalidConfiguration,
            http_status: None,
            user_message: "Die Daylite-URL ist nicht konfiguriert.".to_string(),
            technical_message: "Leere dayliteBaseUrl-Konfiguration".to_string(),
        });
    }

    Ok(trimmed.to_string())
}

pub(super) fn load_daylite_tokens(
    _store: &LocalStore,
) -> Result<DayliteTokenState, DayliteApiError> {
    match crate::secret_manager::get_token("lkr-planner-daylite", "LKR Planner Daylite Token") {
        Ok(json_str) => Ok(serde_json::from_str(&json_str).unwrap_or_default()),
        Err(crate::secret_manager::SecretError::NotFound) => Ok(DayliteTokenState::default()),
        Err(e) => Err(DayliteApiError {
            code: DayliteApiErrorCode::InvalidConfiguration,
            http_status: None,
            user_message: "Auf die Daylite-Zugangsdaten im Keychain konnte nicht zugegriffen werden. Bitte prüfe die Keychain-Berechtigungen.".to_string(),
            technical_message: format!("Keychain-Fehler beim Lesen des Daylite-Tokens: {e}"),
        }),
    }
}


pub(super) fn store_daylite_tokens(_store: &mut LocalStore, token_state: &DayliteTokenState) -> Result<(), DayliteApiError> {
    let json_str = serde_json::to_string(token_state).map_err(|e| DayliteApiError {
        code: DayliteApiErrorCode::ServerError,
        http_status: None,
        user_message: "Token konnten nicht sicher gespeichert werden.".to_string(),
        technical_message: format!("Token serialization failed: {e}"),
    })?;

    crate::secret_manager::set_token("lkr-planner-daylite", "LKR Planner Daylite Token", &json_str).map_err(|e| DayliteApiError {
        code: DayliteApiErrorCode::InvalidConfiguration,
        http_status: None,
        user_message: "Auf den sicheren Speicher konnte nicht zugegriffen werden (Keychain verweigert?).".to_string(),
        technical_message: e.to_string(),
    })
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

    DayliteApiError {
        code,
        http_status: Some(status),
        user_message: user_message.to_string(),
        technical_message: format!(
            "Daylite request failed for {path} with status={status}; body={}",
            truncate_for_log(body)
        ),
    }
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
    let raw_json = serde_json::from_str::<Value>(body).map_err(|error| DayliteApiError {
        code: DayliteApiErrorCode::InvalidResponse,
        http_status: Some(status),
        user_message: "Die Antwort von Daylite konnte nicht verarbeitet werden.".to_string(),
        technical_message: format!(
            "JSON-Parsing für {path} fehlgeschlagen: {error}; body={}",
            truncate_for_log(body)
        ),
    })?;

    serde_json::from_value::<T>(raw_json).map_err(|error| DayliteApiError {
        code: DayliteApiErrorCode::InvalidResponse,
        http_status: Some(status),
        user_message: "Die Antwort von Daylite konnte nicht verarbeitet werden.".to_string(),
        technical_message: format!(
            "JSON-Deserialisierung für {path} fehlgeschlagen: {error}; body={}",
            truncate_for_log(body)
        ),
    })
}

pub(super) fn missing_token_error(user_message: &str, technical_message: &str) -> DayliteApiError {
    DayliteApiError {
        code: DayliteApiErrorCode::MissingToken,
        http_status: None,
        user_message: user_message.to_string(),
        technical_message: technical_message.to_string(),
    }
}

pub(super) fn should_refresh_access_token(token_state: &DayliteTokenState, now_ms: u64) -> bool {
    if token_state.access_token.trim().is_empty() {
        return true;
    }

    match token_state.access_token_expires_at_ms {
        Some(expires_at_ms) => expires_at_ms <= now_ms.saturating_add(10_000),
        None => true,
    }
}

pub(super) fn current_epoch_ms() -> Result<u64, DayliteApiError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DayliteApiError {
            code: DayliteApiErrorCode::RequestFailed,
            http_status: None,
            user_message: "Die aktuelle Systemzeit konnte nicht gelesen werden.".to_string(),
            technical_message: format!("Systemzeitfehler: {error}"),
        })?;

    u64::try_from(duration.as_millis()).map_err(|error| DayliteApiError {
        code: DayliteApiErrorCode::RequestFailed,
        http_status: None,
        user_message: "Die aktuelle Systemzeit konnte nicht gelesen werden.".to_string(),
        technical_message: format!("Zeitstempel-Konvertierung fehlgeschlagen: {error}"),
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

fn map_store_error(error: local_store::StoreError) -> DayliteApiError {
    DayliteApiError {
        code: DayliteApiErrorCode::InvalidConfiguration,
        http_status: None,
        user_message: error.user_message,
        technical_message: error.technical_message,
    }
}
