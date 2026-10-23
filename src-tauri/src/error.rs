use serde::{Deserialize, Serialize};
use specta::Type;

/// Only `user_message` is meant for the user.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: AppErrorCode,
    pub http_status: Option<u16>,
    pub user_message: String,
    pub technical_message: String,
}

impl AppError {
    pub fn new(
        code: AppErrorCode,
        user_message: impl Into<String>,
        technical_message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            http_status: None,
            user_message: user_message.into(),
            technical_message: technical_message.into(),
        }
    }

    pub fn with_http_status(mut self, http_status: Option<u16>) -> Self {
        self.http_status = http_status;
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppErrorCode {
    CorruptFile,
    DayliteSyncFailed,
    InvalidConfiguration,
    InvalidResponse,
    KeychainError,
    MissingCredentials,
    MissingFields,
    MissingToken,
    NetworkError,
    NotFound,
    RateLimited,
    ReadFailed,
    RequestFailed,
    ServerError,
    Timeout,
    TokenRefreshFailed,
    Unauthorized,
    WriteFailed,
}
