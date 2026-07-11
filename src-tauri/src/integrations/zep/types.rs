use crate::integrations::local_store::StoreError;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepError {
    pub code: ZepErrorCode,
    pub user_message: String,
    pub technical_message: String,
}

impl ZepError {
    pub(crate) fn new(
        code: ZepErrorCode,
        user_message: impl Into<String>,
        technical_message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            user_message: user_message.into(),
            technical_message: technical_message.into(),
        }
    }
}

impl From<StoreError> for ZepError {
    fn from(error: StoreError) -> Self {
        ZepError::new(
            ZepErrorCode::InvalidConfiguration,
            error.user_message,
            error.technical_message,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZepErrorCode {
    KeychainError,
    MissingCredentials,
    Unauthorized,
    NotFound,
    NetworkError,
    InvalidResponse,
    InvalidConfiguration,
    DayliteSyncFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCalendar {
    pub display_name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCredentialTestResult {
    pub calendar_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCredentialsInfo {
    pub root_url: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCalendarTestResult {
    pub success: bool,
    pub timestamp: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IcalSource {
    Primary,
    Absence,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ZepStoredCredentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl std::fmt::Debug for ZepStoredCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZepStoredCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}
