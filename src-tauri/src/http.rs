use std::sync::OnceLock;
use std::time::Duration;
use tauri_plugin_http::reqwest;

/// Cloning shares the connection pool; a request that needs a shorter limit sets its own timeout.
#[allow(
    clippy::disallowed_methods,
    reason = "the one place that builds the shared client"
)]
pub(crate) fn shared_client() -> Result<reqwest::Client, String> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))
        })
        .clone()
}
