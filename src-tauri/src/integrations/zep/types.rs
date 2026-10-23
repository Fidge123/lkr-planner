pub use crate::error::{AppError as ZepError, AppErrorCode as ZepErrorCode};
use serde::{Deserialize, Serialize};
use specta::Type;

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

#[derive(Clone, Serialize, Deserialize)]
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
