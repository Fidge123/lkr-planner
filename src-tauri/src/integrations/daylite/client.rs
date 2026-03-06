use super::shared::{normalize_base_url, DayliteApiError, DayliteApiErrorCode};
#[cfg(test)]
use crate::integrations::http_record_replay::{
    RecordReplayConfig, RecordedInteraction, RecordedRequest, RecordedResponse, VcrMode,
};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::header::AUTHORIZATION;

pub(super) struct DayliteApiClient {
    transport: Arc<dyn DayliteHttpTransport>,
}

impl DayliteApiClient {
    pub(super) fn new(base_url: &str) -> Result<Self, DayliteApiError> {
        let transport = ReqwestTransport::new(base_url)?;
        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    #[cfg(test)]
    pub(super) fn with_transport(transport: Arc<dyn DayliteHttpTransport>) -> Self {
        Self { transport }
    }

    pub(super) async fn send_request(
        &self,
        method: DayliteHttpMethod,
        path: &str,
        query: Vec<(String, String)>,
        body: Option<Value>,
        access_token: Option<String>,
    ) -> Result<DayliteHttpResponse, DayliteApiError> {
        self.transport
            .send(DayliteHttpRequest {
                method,
                path: path.to_string(),
                query,
                body,
                access_token,
            })
            .await
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
}

#[derive(Debug, Clone)]
pub(super) struct DayliteHttpRequest {
    pub method: DayliteHttpMethod,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DayliteHttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone)]
struct ReqwestTransport {
    base_url: String,
    http_client: reqwest::Client,
    #[cfg(test)]
    record_replay: Option<RecordReplayConfig>,
}

impl ReqwestTransport {
    fn new(base_url: &str) -> Result<Self, DayliteApiError> {
        let normalized_base_url = normalize_base_url(base_url)?;
        let http_client = reqwest::Client::builder()
            .build()
            .map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: None,
                user_message: "Die Verbindung zu Daylite konnte nicht aufgebaut werden."
                    .to_string(),
                technical_message: format!("HTTP-Client konnte nicht erstellt werden: {error}"),
            })?;

        Ok(Self {
            base_url: normalized_base_url,
            http_client,
            #[cfg(test)]
            record_replay: None,
        })
    }

    #[cfg(test)]
    fn new_with_record_replay(
        base_url: &str,
        record_replay: RecordReplayConfig,
    ) -> Result<Self, DayliteApiError> {
        let mut transport = Self::new(base_url)?;
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
                .map_err(|error| DayliteApiError {
                    code: DayliteApiErrorCode::InvalidConfiguration,
                    http_status: None,
                    user_message: "Die Daylite-URL ist ungültig konfiguriert.".to_string(),
                    technical_message: format!("URL konnte nicht geparst werden: {error}"),
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
            };

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

            let response = builder.send().await.map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: None,
                user_message: "Die Anfrage an Daylite ist fehlgeschlagen.".to_string(),
                technical_message: format!("Netzwerkfehler bei {}: {error}", request.path),
            })?;

            let status = response.status().as_u16();
            let body = response.text().await.map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: Some(status),
                user_message: "Die Antwort von Daylite konnte nicht gelesen werden.".to_string(),
                technical_message: format!("Antworttext konnte nicht gelesen werden: {error}"),
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
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    #[test]
    fn records_sanitized_cassette_in_record_mode() {
        let cassette_path = cassette_path("daylite-record-mode-generated.json");
        remove_cassette_if_present(&cassette_path);

        let config = RecordReplayConfig::new(cassette_path.clone(), VcrMode::Record);
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
    }

    #[test]
    fn replays_recorded_response_without_network_call() {
        let cassette_path = cassette_path("daylite-client-replay.json");
        let transport = ReqwestTransport::new_with_record_replay(
            "http://127.0.0.1:9",
            RecordReplayConfig::new(cassette_path, VcrMode::Replay),
        )
        .expect("replay transport should be created");
        let request = DayliteHttpRequest {
            method: DayliteHttpMethod::Get,
            path: "/projects".to_string(),
            query: vec![("full-records".to_string(), "true".to_string())],
            body: None,
            access_token: Some("ignored-in-replay".to_string()),
        };

        let started_at = Instant::now();
        let first = tauri::async_runtime::block_on(async { transport.send(request.clone()).await })
            .expect("first replay should succeed");
        let second = tauri::async_runtime::block_on(async { transport.send(request).await })
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

    fn cassette_path(file_name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/cassettes")
            .join(file_name)
    }

    fn remove_cassette_if_present(path: &Path) {
        let _ = fs::remove_file(path);
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }
}
