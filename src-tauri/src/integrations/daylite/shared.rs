use super::super::local_store::{self, LocalStore};
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

pub(super) fn load_store_or_error(app: tauri::AppHandle) -> Result<LocalStore, AppError> {
    local_store::load_local_store(app).map_err(map_store_error)
}

pub(super) fn update_store_or_error(
    app: &tauri::AppHandle,
    change: impl FnOnce(&mut LocalStore),
) -> Result<(), AppError> {
    local_store::update_local_store(app, change).map_err(map_store_error)
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
