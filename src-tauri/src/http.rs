use crate::error::{AppError, AppErrorCode};
use std::sync::OnceLock;
use std::time::Duration;
use tauri_plugin_http::reqwest;

/// Cloning shares the connection pool; a request that needs a shorter limit sets its own timeout.
#[allow(
    clippy::disallowed_methods,
    reason = "the one place that builds the shared client"
)]
pub(crate) fn shared_client() -> Result<reqwest::Client, AppError> {
    static CLIENT: OnceLock<Result<reqwest::Client, AppError>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|error| {
                    AppError::new(
                        AppErrorCode::NetworkError,
                        "Die Netzwerkverbindung konnte nicht vorbereitet werden.",
                        format!("HTTP client could not be built: {error}"),
                    )
                })
        })
        .clone()
}
