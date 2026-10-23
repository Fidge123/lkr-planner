use super::shared::{
    normalize_base_url, normalize_http_error, DayliteApiError, DayliteApiErrorCode,
};
#[cfg(test)]
use crate::integrations::http_record_replay::{
    cassette_path_for_test, RecordReplayConfig, RecordedInteraction, RecordedRequest,
    RecordedResponse, VcrMode,
};
use crate::integrations::telemetry::events::{ErrorCode, Integration, Operation};
use crate::integrations::telemetry::observe::{observe_with, TelemetryError};
use crate::integrations::telemetry::recorder::TelemetryRecorder;
use crate::integrations::telemetry::state::TelemetryState;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::header::AUTHORIZATION;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(7);

pub(super) struct DayliteApiClient {
    transport: Box<dyn DayliteHttpTransport>,
    telemetry: Option<std::sync::Arc<TelemetryRecorder>>,
}

impl DayliteApiClient {
    pub(super) fn new(base_url: &str) -> Result<Self, DayliteApiError> {
        Self::with_timeout(base_url, DEFAULT_TIMEOUT)
    }

    pub(super) fn with_timeout(base_url: &str, timeout: Duration) -> Result<Self, DayliteApiError> {
        let transport = ReqwestTransport::new(base_url, timeout)?;
        Ok(Self {
            transport: Box::new(transport),
            telemetry: None,
        })
    }

    pub(super) fn with_telemetry(mut self, app: &tauri::AppHandle) -> Self {
        self.telemetry = app
            .try_state::<TelemetryState>()
            .map(|state| std::sync::Arc::clone(state.recorder()));

        self
    }

    #[cfg(test)]
    pub(super) fn with_transport(transport: Box<dyn DayliteHttpTransport>) -> Self {
        Self {
            transport,
            telemetry: None,
        }
    }

    /// Unreachable port, so a cassette miss fails fast instead of reaching the live API.
    #[cfg(test)]
    pub(super) fn with_replay_cassette(cassette_file_name: &str) -> Result<Self, DayliteApiError> {
        Self::with_record_replay(
            "http://127.0.0.1:9",
            RecordReplayConfig::new(cassette_path_for_test(cassette_file_name), VcrMode::Replay),
        )
    }

    #[cfg(test)]
    pub(super) fn with_env_cassette(
        base_url: &str,
        cassette_file_name: &str,
    ) -> Result<Self, DayliteApiError> {
        Self::with_record_replay(
            base_url,
            RecordReplayConfig::from_env(cassette_path_for_test(cassette_file_name)),
        )
    }

    pub(super) async fn send_request(
        &self,
        request: DayliteHttpRequest,
    ) -> Result<DayliteHttpResponse, DayliteApiError> {
        let path = request.path.clone();
        let result = observe_with(
            self.telemetry.as_deref(),
            Operation::DayliteRequest,
            Integration::Daylite,
            async {
                let response = self
                    .transport
                    .send(request)
                    .await
                    .map_err(RequestFailure::Transport)?;
                if (200..300).contains(&response.status) {
                    Ok(response)
                } else {
                    Err(RequestFailure::Status {
                        error: normalize_http_error(response.status, &response.body, &path),
                        response,
                    })
                }
            },
        )
        .await;

        match result {
            Ok(response) | Err(RequestFailure::Status { response, .. }) => Ok(response),
            Err(RequestFailure::Transport(error)) => Err(error),
        }
    }

    #[cfg(test)]
    fn with_record_replay(
        base_url: &str,
        config: RecordReplayConfig,
    ) -> Result<Self, DayliteApiError> {
        Ok(Self {
            transport: Box::new(ReqwestTransport::new_with_record_replay(base_url, config)?),
            telemetry: None,
        })
    }
}

/// Callers map unsuccessful statuses themselves, so `Status` is only a failure for telemetry.
enum RequestFailure {
    Transport(DayliteApiError),
    Status {
        error: DayliteApiError,
        response: DayliteHttpResponse,
    },
}

impl RequestFailure {
    fn error(&self) -> &DayliteApiError {
        match self {
            Self::Transport(error) | Self::Status { error, .. } => error,
        }
    }
}

impl TelemetryError for RequestFailure {
    fn error_code(&self) -> ErrorCode {
        self.error().error_code()
    }

    fn technical_message(&self) -> Option<&str> {
        self.error().technical_message()
    }

    fn http_status(&self) -> Option<u16> {
        self.error().http_status()
    }
}

pub(super) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub(super) trait DayliteHttpTransport: Send + Sync {
    fn send<'a>(
        &'a self,
        request: DayliteHttpRequest,
    ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DayliteHttpMethod {
    Get,
    Post,
    Patch,
    Delete,
}

#[derive(Debug, Clone)]
pub(super) struct DayliteHttpRequest {
    pub method: DayliteHttpMethod,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub access_token: Option<String>,
}

impl DayliteHttpRequest {
    pub(super) fn new(method: DayliteHttpMethod, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            query: Vec::new(),
            body: None,
            access_token: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DayliteHttpResponse {
    pub status: u16,
    pub body: String,
}

/// Cloning a reqwest client shares its connection pool rather than copying it.
fn shared_http_client() -> Result<reqwest::Client, DayliteApiError> {
    static CLIENT: std::sync::OnceLock<Result<reqwest::Client, String>> =
        std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .build()
                .map_err(|error| format!("HTTP-Client konnte nicht erstellt werden: {error}"))
        })
        .clone()
        .map_err(|technical_message| {
            DayliteApiError::new(
                DayliteApiErrorCode::RequestFailed,
                "Die Verbindung zu Daylite konnte nicht aufgebaut werden.",
                technical_message,
            )
        })
}

#[derive(Debug, Clone)]
struct ReqwestTransport {
    base_url: String,
    http_client: reqwest::Client,
    timeout: Duration,
    #[cfg(test)]
    record_replay: Option<RecordReplayConfig>,
}

impl ReqwestTransport {
    fn new(base_url: &str, timeout: Duration) -> Result<Self, DayliteApiError> {
        Ok(Self {
            base_url: normalize_base_url(base_url)?,
            http_client: shared_http_client()?,
            timeout,
            #[cfg(test)]
            record_replay: None,
        })
    }

    #[cfg(test)]
    fn new_with_record_replay(
        base_url: &str,
        record_replay: RecordReplayConfig,
    ) -> Result<Self, DayliteApiError> {
        let mut transport = Self::new(base_url, DEFAULT_TIMEOUT)?;
        transport.record_replay = Some(record_replay);
        Ok(transport)
    }
}

impl DayliteHttpTransport for ReqwestTransport {
    fn send<'a>(
        &'a self,
        request: DayliteHttpRequest,
    ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>> {
        Box::pin(async move {
            #[cfg(test)]
            let recorded_request = self
                .record_replay
                .as_ref()
                .map(|_| to_recorded_request(&request));

            #[cfg(test)]
            if let Some(record_replay) = &self.record_replay {
                if record_replay.mode() == VcrMode::Replay {
                    let response = record_replay
                        .replay(
                            recorded_request
                                .as_ref()
                                .expect("recorded request should exist"),
                        )
                        .map_err(|error| DayliteApiError {
                            code: DayliteApiErrorCode::RequestFailed,
                            http_status: None,
                            user_message:
                                "Die Testkassette fuer Daylite konnte nicht geladen werden."
                                    .to_string(),
                            technical_message: format!(
                                "Cassette replay for {} failed: {error}",
                                request.path
                            ),
                        })?
                        .ok_or_else(|| DayliteApiError {
                            code: DayliteApiErrorCode::RequestFailed,
                            http_status: None,
                            user_message: "Die passende Daylite-Testkassette wurde nicht gefunden."
                                .to_string(),
                            technical_message: format!(
                                "No cassette interaction matched {} {}",
                                request.method.as_str(),
                                request.path
                            ),
                        })?;

                    return Ok(DayliteHttpResponse {
                        status: response.status,
                        body: response.body,
                    });
                }
            }

            let mut url = reqwest::Url::parse(&format!("{}{}", self.base_url, request.path))
                .map_err(|error| {
                    DayliteApiError::new(
                        DayliteApiErrorCode::InvalidConfiguration,
                        "Die Daylite-URL ist ungültig konfiguriert.",
                        format!("URL konnte nicht geparst werden: {error}"),
                    )
                })?;

            {
                let mut query_pairs = url.query_pairs_mut();
                for (key, value) in &request.query {
                    query_pairs.append_pair(key, value);
                }
            }

            let mut builder = match request.method {
                DayliteHttpMethod::Get => self.http_client.get(url),
                DayliteHttpMethod::Post => self.http_client.post(url),
                DayliteHttpMethod::Patch => self.http_client.patch(url),
                DayliteHttpMethod::Delete => self.http_client.delete(url),
            }
            .timeout(self.timeout);

            if let Some(access_token) = request.access_token {
                if !access_token.trim().is_empty() {
                    builder = builder.header(AUTHORIZATION, format!("Bearer {access_token}"));
                }
            }

            if let Some(body) = request.body {
                builder = builder
                    .header("content-type", "application/json")
                    .body(body.to_string());
            }

            let response = builder.send().await.map_err(|error| {
                if error.is_timeout() {
                    DayliteApiError::new(
                        DayliteApiErrorCode::Timeout,
                        "Zeitüberschreitung bei der Daylite-Anfrage.",
                        format!("Zeitüberschreitung bei {}: {error}", request.path),
                    )
                } else {
                    DayliteApiError::new(
                        DayliteApiErrorCode::RequestFailed,
                        "Die Anfrage an Daylite ist fehlgeschlagen.",
                        format!("Netzwerkfehler bei {}: {error}", request.path),
                    )
                }
            })?;

            let status = response.status().as_u16();
            let body = response.text().await.map_err(|error| {
                DayliteApiError::new(
                    DayliteApiErrorCode::RequestFailed,
                    "Die Antwort von Daylite konnte nicht gelesen werden.",
                    format!("Antworttext konnte nicht gelesen werden: {error}"),
                )
                .with_http_status(Some(status))
            })?;

            #[cfg(test)]
            if let Some(record_replay) = &self.record_replay {
                if record_replay.mode() == VcrMode::Record {
                    record_replay
                        .record(RecordedInteraction {
                            request: recorded_request.expect("recorded request should exist"),
                            response: RecordedResponse {
                                status,
                                body: body.clone(),
                            },
                        })
                        .map_err(|error| DayliteApiError {
                            code: DayliteApiErrorCode::RequestFailed,
                            http_status: None,
                            user_message:
                                "Die Daylite-Testkassette konnte nicht gespeichert werden."
                                    .to_string(),
                            technical_message: format!(
                                "Cassette recording for {} failed: {error}",
                                request.path
                            ),
                        })?;
                }
            }

            Ok(DayliteHttpResponse { status, body })
        })
    }
}

#[cfg(test)]
impl DayliteHttpMethod {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

#[cfg(test)]
fn to_recorded_request(request: &DayliteHttpRequest) -> RecordedRequest {
    RecordedRequest {
        method: request.method.as_str().to_string(),
        path: request.path.clone(),
        query: request.query.clone(),
        body: request.body.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::telemetry::test_support::{enabled_recorder, recorded};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    #[test]
    fn records_sanitized_cassette_in_record_mode() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        let cassette_path = cassette_path("daylite-record-mode-generated.json");
        remove_cassette_if_present(&cassette_path);
        unsafe {
            std::env::set_var("VCR_MODE", "record");
        }

        let config = RecordReplayConfig::from_env(cassette_path.clone());
        config
            .record(RecordedInteraction {
                request: to_recorded_request(&DayliteHttpRequest {
                    method: DayliteHttpMethod::Get,
                    path: "/projects".to_string(),
                    query: vec![("full-records".to_string(), "true".to_string())],
                    body: None,
                    access_token: Some("top-secret-token".to_string()),
                }),
                response: RecordedResponse {
                    status: 200,
                    body: r#"{"data":[{"name":"Recorded Project"}]}"#.to_string(),
                },
            })
            .expect("cassette should be written in record mode");

        let cassette = fs::read_to_string(&cassette_path).expect("cassette should be written");
        assert!(!cassette.contains("Authorization"));
        assert!(!cassette.contains("Cookie"));
        assert!(!cassette.contains("x-api-key"));
        assert!(!cassette.contains("top-secret-token"));

        remove_cassette_if_present(&cassette_path);
        unsafe {
            std::env::remove_var("VCR_MODE");
        }
    }

    #[tokio::test]
    async fn a_client_with_its_own_timeout_gives_up_after_it() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind silent server");
        let base_url = format!(
            "http://{}",
            listener.local_addr().expect("silent server addr")
        );
        tokio::spawn(async move {
            let mut unanswered = Vec::new();
            while let Ok((stream, _)) = listener.accept().await {
                unanswered.push(stream);
            }
        });
        let client = DayliteApiClient::with_timeout(&base_url, Duration::from_millis(100))
            .expect("client should be created");

        let result = tokio::time::timeout(
            Duration::from_secs(2),
            client.send_request(DayliteHttpRequest::new(DayliteHttpMethod::Get, "/projects")),
        )
        .await
        .expect("the client's timeout should end the request first");

        assert_eq!(
            result.expect_err("the server never answers").code,
            DayliteApiErrorCode::Timeout
        );
    }

    #[tokio::test]
    async fn replays_recorded_response_without_network_call() {
        // The env lock covers only the VCR_MODE mutation and read, so it is dropped before the awaits.
        let transport = {
            let _guard = env_lock().lock().expect("env lock should not be poisoned");
            let cassette_path = cassette_path("daylite-client-replay.json");
            unsafe {
                std::env::remove_var("VCR_MODE");
            }
            ReqwestTransport::new_with_record_replay(
                "http://127.0.0.1:9",
                RecordReplayConfig::from_env(cassette_path),
            )
            .expect("replay transport should be created")
        };
        let request = DayliteHttpRequest {
            method: DayliteHttpMethod::Get,
            path: "/projects".to_string(),
            query: vec![("full-records".to_string(), "true".to_string())],
            body: None,
            access_token: Some("ignored-in-replay".to_string()),
        };

        let started_at = Instant::now();
        let first = transport
            .send(request.clone())
            .await
            .expect("first replay should succeed");
        let second = transport
            .send(request)
            .await
            .expect("second replay should succeed");

        assert_eq!(first.status, 200);
        assert_eq!(first, second);
        assert!(started_at.elapsed() < Duration::from_millis(200));
    }

    #[test]
    fn derives_vcr_mode_from_environment() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");

        unsafe {
            std::env::set_var("VCR_MODE", "record");
        }
        assert_eq!(VcrMode::from_env(), VcrMode::Record);

        unsafe {
            std::env::set_var("VCR_MODE", "unexpected");
        }
        assert_eq!(VcrMode::from_env(), VcrMode::Replay);

        unsafe {
            std::env::remove_var("VCR_MODE");
        }
        assert_eq!(VcrMode::from_env(), VcrMode::Replay);
    }

    #[tokio::test]
    async fn an_unsuccessful_status_is_recorded_as_a_failure_but_still_returned() {
        let (recorder, mut receiver) = enabled_recorder();
        let client = DayliteApiClient {
            transport: Box::new(FixedResponseTransport(DayliteHttpResponse {
                status: 404,
                body: "not found".to_string(),
            })),
            telemetry: Some(recorder),
        };

        let response = client
            .send_request(DayliteHttpRequest::new(DayliteHttpMethod::Get, "/projects"))
            .await
            .expect("an unsuccessful status should still be returned");

        assert_eq!(response.status, 404);
        assert_eq!(response.body, "not found");
        let events = recorded(&mut receiver);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name(), "operation_completed");
        assert_eq!(events[0].properties()["success"], false);
        assert_eq!(events[0].properties()["http_status"], 404);
        assert_eq!(events[1].name(), "error_occurred");
        assert_eq!(events[1].properties()["code"], "REQUEST_FAILED");
        assert_eq!(events[1].properties()["http_status"], 404);
    }

    #[tokio::test]
    async fn a_server_error_is_recorded_with_its_mapped_code() {
        let (recorder, mut receiver) = enabled_recorder();
        let client = DayliteApiClient {
            transport: Box::new(FixedResponseTransport(DayliteHttpResponse {
                status: 500,
                body: String::new(),
            })),
            telemetry: Some(recorder),
        };

        let response = client
            .send_request(DayliteHttpRequest::new(DayliteHttpMethod::Get, "/projects"))
            .await
            .expect("a server error status should still be returned");

        assert_eq!(response.status, 500);
        let events = recorded(&mut receiver);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].properties()["success"], false);
        assert_eq!(events[0].properties()["http_status"], 500);
        assert_eq!(events[1].properties()["code"], "SERVER_ERROR");
        assert_eq!(events[1].properties()["http_status"], 500);
    }

    #[tokio::test]
    async fn a_successful_status_is_recorded_as_a_single_completion() {
        let (recorder, mut receiver) = enabled_recorder();
        let client = DayliteApiClient {
            transport: Box::new(FixedResponseTransport(DayliteHttpResponse {
                status: 200,
                body: "{}".to_string(),
            })),
            telemetry: Some(recorder),
        };

        let response = client
            .send_request(DayliteHttpRequest::new(DayliteHttpMethod::Get, "/projects"))
            .await
            .expect("a successful request should be returned");

        assert_eq!(response.status, 200);
        let events = recorded(&mut receiver);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name(), "operation_completed");
        assert_eq!(events[0].properties()["success"], true);
        assert!(!events[0].properties().contains_key("http_status"));
    }

    #[tokio::test]
    async fn separate_clients_reuse_one_connection() {
        let (base_url, connections) =
            crate::integrations::test_http::spawn_keep_alive_server().await;

        for _ in 0..2 {
            let client = DayliteApiClient::new(&base_url).expect("client should be created");
            let response = client
                .send_request(DayliteHttpRequest::new(DayliteHttpMethod::Get, "/projects"))
                .await
                .expect("request should succeed");
            assert_eq!(response.status, 200);
        }

        assert_eq!(connections.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    struct FixedResponseTransport(DayliteHttpResponse);

    impl DayliteHttpTransport for FixedResponseTransport {
        fn send<'a>(
            &'a self,
            _request: DayliteHttpRequest,
        ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>> {
            let response = self.0.clone();
            Box::pin(async move { Ok(response) })
        }
    }

    fn cassette_path(file_name: &str) -> PathBuf {
        cassette_path_for_test(file_name)
    }

    fn remove_cassette_if_present(path: &Path) {
        let _ = fs::remove_file(path);
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }
}
