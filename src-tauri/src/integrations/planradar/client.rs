use super::shared::{normalize_base_url, PlanradarApiError, PlanradarApiErrorCode};
#[cfg(test)]
use crate::integrations::http_record_replay::{
    RecordReplayConfig, RecordedInteraction, RecordedRequest, RecordedResponse, VcrMode,
};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tauri_plugin_http::reqwest;

/// Header carrying the static Planradar personal access token, per the Open API spec
/// (`securityDefinitions.apiKey` → `X-PlanRadar-API-Key`).
const PLANRADAR_API_KEY_HEADER: &str = "X-PlanRadar-API-Key";

/// Retry behavior for transient Planradar failures (rate limiting and 5xx/network errors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
}

impl RetryPolicy {
    /// Production default: a handful of retries with exponential backoff (200ms, 400ms, 800ms).
    pub(super) fn standard() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 200,
        }
    }

    #[cfg(test)]
    pub(super) fn none() -> Self {
        Self {
            max_retries: 0,
            base_delay_ms: 0,
        }
    }

    #[cfg(test)]
    pub(super) fn immediate(max_retries: u32) -> Self {
        Self {
            max_retries,
            base_delay_ms: 0,
        }
    }
}

pub(super) struct PlanradarApiClient {
    transport: Box<dyn PlanradarHttpTransport>,
    retry: RetryPolicy,
}

impl PlanradarApiClient {
    pub(super) fn new(base_url: &str) -> Result<Self, PlanradarApiError> {
        let transport = ReqwestTransport::new(base_url)?;
        Ok(Self {
            transport: Box::new(transport),
            retry: RetryPolicy::standard(),
        })
    }

    #[cfg(test)]
    pub(super) fn with_transport(transport: Box<dyn PlanradarHttpTransport>) -> Self {
        Self {
            transport,
            retry: RetryPolicy::none(),
        }
    }

    #[cfg(test)]
    pub(super) fn with_transport_and_retry(
        transport: Box<dyn PlanradarHttpTransport>,
        retry: RetryPolicy,
    ) -> Self {
        Self { transport, retry }
    }

    #[cfg(test)]
    pub(super) fn with_replay_cassette(
        cassette_file_name: &str,
    ) -> Result<Self, PlanradarApiError> {
        let transport = ReqwestTransport::new_with_record_replay(
            "http://127.0.0.1:9",
            RecordReplayConfig::new(cassette_path_for_test(cassette_file_name), VcrMode::Replay),
        )?;

        Ok(Self {
            transport: Box::new(transport),
            retry: RetryPolicy::none(),
        })
    }

    #[cfg(test)]
    pub(super) fn with_env_cassette(
        base_url: &str,
        cassette_file_name: &str,
    ) -> Result<Self, PlanradarApiError> {
        let transport = ReqwestTransport::new_with_record_replay(
            base_url,
            RecordReplayConfig::from_env(cassette_path_for_test(cassette_file_name)),
        )?;

        Ok(Self {
            transport: Box::new(transport),
            retry: RetryPolicy::standard(),
        })
    }

    /// Sends a request, retrying transient failures (rate limiting, 5xx, and network errors)
    /// with exponential backoff according to the configured [`RetryPolicy`].
    pub(super) async fn send_request(
        &self,
        method: PlanradarHttpMethod,
        path: &str,
        query: Vec<(String, String)>,
        body: Option<Value>,
        api_key: Option<String>,
    ) -> Result<PlanradarHttpResponse, PlanradarApiError> {
        let mut attempt = 0;
        loop {
            let request = PlanradarHttpRequest {
                method,
                path: path.to_string(),
                query: query.clone(),
                body: body.clone(),
                api_key: api_key.clone(),
            };

            let result = self.transport.send(request).await;
            let should_retry = attempt < self.retry.max_retries
                && match &result {
                    Ok(response) => is_retryable_status(response.status),
                    Err(error) => is_retryable_error(error),
                };

            if !should_retry {
                return result;
            }

            backoff_delay(self.retry.base_delay_ms, attempt).await;
            attempt += 1;
        }
    }
}

fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..=599).contains(&status)
}

fn is_retryable_error(error: &PlanradarApiError) -> bool {
    matches!(
        error.code,
        PlanradarApiErrorCode::Timeout
            | PlanradarApiErrorCode::ServerError
            | PlanradarApiErrorCode::RequestFailed
    )
}

async fn backoff_delay(base_delay_ms: u64, attempt: u32) {
    if base_delay_ms == 0 {
        return;
    }

    let multiplier = 2u64.saturating_pow(attempt);
    let delay = base_delay_ms.saturating_mul(multiplier);
    tokio::time::sleep(Duration::from_millis(delay)).await;
}

pub(super) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub(super) trait PlanradarHttpTransport: Send + Sync {
    fn send<'a>(
        &'a self,
        request: PlanradarHttpRequest,
    ) -> BoxFuture<'a, Result<PlanradarHttpResponse, PlanradarApiError>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlanradarHttpMethod {
    Get,
    Post,
    Put,
}

#[derive(Debug, Clone)]
pub(super) struct PlanradarHttpRequest {
    pub method: PlanradarHttpMethod,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlanradarHttpResponse {
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
    fn new(base_url: &str) -> Result<Self, PlanradarApiError> {
        let normalized_base_url = normalize_base_url(base_url)?;
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|error| {
                PlanradarApiError::new(
                    PlanradarApiErrorCode::RequestFailed,
                    None,
                    "Die Verbindung zu Planradar konnte nicht aufgebaut werden.",
                    format!("HTTP-Client konnte nicht erstellt werden: {error}"),
                )
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
    ) -> Result<Self, PlanradarApiError> {
        let mut transport = Self::new(base_url)?;
        transport.record_replay = Some(record_replay);
        Ok(transport)
    }
}

impl PlanradarHttpTransport for ReqwestTransport {
    fn send<'a>(
        &'a self,
        request: PlanradarHttpRequest,
    ) -> BoxFuture<'a, Result<PlanradarHttpResponse, PlanradarApiError>> {
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
                        .map_err(|error| {
                            PlanradarApiError::new(
                                PlanradarApiErrorCode::RequestFailed,
                                None,
                                "Die Testkassette für Planradar konnte nicht geladen werden.",
                                format!("Cassette replay for {} failed: {error}", request.path),
                            )
                        })?
                        .ok_or_else(|| {
                            PlanradarApiError::new(
                                PlanradarApiErrorCode::RequestFailed,
                                None,
                                "Die passende Planradar-Testkassette wurde nicht gefunden.",
                                format!(
                                    "No cassette interaction matched {} {}",
                                    request.method.as_str(),
                                    request.path
                                ),
                            )
                        })?;

                    return Ok(PlanradarHttpResponse {
                        status: response.status,
                        body: response.body,
                    });
                }
            }

            let mut url = reqwest::Url::parse(&format!("{}{}", self.base_url, request.path))
                .map_err(|error| {
                    PlanradarApiError::new(
                        PlanradarApiErrorCode::InvalidConfiguration,
                        None,
                        "Die Planradar-URL ist ungültig konfiguriert.",
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
                PlanradarHttpMethod::Get => self.http_client.get(url),
                PlanradarHttpMethod::Post => self.http_client.post(url),
                PlanradarHttpMethod::Put => self.http_client.put(url),
            };

            if let Some(api_key) = request.api_key {
                if !api_key.trim().is_empty() {
                    builder = builder.header(PLANRADAR_API_KEY_HEADER, api_key);
                }
            }

            if let Some(body) = request.body {
                builder = builder
                    .header("content-type", "application/json")
                    .body(body.to_string());
            }

            let response = builder.send().await.map_err(|error| {
                if error.is_timeout() {
                    PlanradarApiError::new(
                        PlanradarApiErrorCode::Timeout,
                        None,
                        "Zeitüberschreitung bei der Planradar-Anfrage.",
                        format!("Zeitüberschreitung bei {}: {error}", request.path),
                    )
                } else {
                    PlanradarApiError::new(
                        PlanradarApiErrorCode::RequestFailed,
                        None,
                        "Die Anfrage an Planradar ist fehlgeschlagen.",
                        format!("Netzwerkfehler bei {}: {error}", request.path),
                    )
                }
            })?;

            let status = response.status().as_u16();
            let body = response.text().await.map_err(|error| {
                PlanradarApiError::new(
                    PlanradarApiErrorCode::RequestFailed,
                    Some(status),
                    "Die Antwort von Planradar konnte nicht gelesen werden.",
                    format!("Antworttext konnte nicht gelesen werden: {error}"),
                )
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
                        .map_err(|error| {
                            PlanradarApiError::new(
                                PlanradarApiErrorCode::RequestFailed,
                                None,
                                "Die Planradar-Testkassette konnte nicht gespeichert werden.",
                                format!("Cassette recording for {} failed: {error}", request.path),
                            )
                        })?;
                }
            }

            Ok(PlanradarHttpResponse { status, body })
        })
    }
}

#[cfg(test)]
impl PlanradarHttpMethod {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
        }
    }
}

#[cfg(test)]
fn to_recorded_request(request: &PlanradarHttpRequest) -> RecordedRequest {
    RecordedRequest {
        method: request.method.as_str().to_string(),
        path: request.path.clone(),
        query: request.query.clone(),
        body: request.body.clone(),
    }
}

#[cfg(test)]
fn cassette_path_for_test(file_name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/cassettes")
        .join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    #[derive(Clone)]
    pub(crate) struct MockTransport {
        responses: Arc<Mutex<VecDeque<Result<PlanradarHttpResponse, PlanradarApiError>>>>,
        requests: Arc<Mutex<Vec<PlanradarHttpRequest>>>,
    }

    impl MockTransport {
        pub(crate) fn new(
            responses: Vec<Result<PlanradarHttpResponse, PlanradarApiError>>,
        ) -> Self {
            Self {
                responses: Arc::new(Mutex::new(VecDeque::from(responses))),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub(crate) fn requests(&self) -> Vec<PlanradarHttpRequest> {
            self.requests
                .lock()
                .expect("request lock should succeed")
                .clone()
        }
    }

    impl PlanradarHttpTransport for MockTransport {
        fn send<'a>(
            &'a self,
            request: PlanradarHttpRequest,
        ) -> BoxFuture<'a, Result<PlanradarHttpResponse, PlanradarApiError>> {
            Box::pin(async move {
                self.requests
                    .lock()
                    .expect("request lock should succeed")
                    .push(request);

                self.responses
                    .lock()
                    .expect("response lock should succeed")
                    .pop_front()
                    .expect("mock should contain enough responses")
            })
        }
    }

    fn mock_response(status: u16, body: &str) -> PlanradarHttpResponse {
        PlanradarHttpResponse {
            status,
            body: body.to_string(),
        }
    }

    #[test]
    fn send_request_retries_rate_limit_then_succeeds() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(429, "rate limited")),
                Ok(mock_response(200, r#"{"ok":true}"#)),
            ]);
            let client = PlanradarApiClient::with_transport_and_retry(
                Box::new(transport.clone()),
                RetryPolicy::immediate(3),
            );

            let started_at = Instant::now();
            let response = client
                .send_request(PlanradarHttpMethod::Get, "/x", vec![], None, None)
                .await
                .expect("retried request should succeed");

            assert_eq!(response.status, 200);
            assert_eq!(transport.requests().len(), 2);
            assert!(started_at.elapsed() < Duration::from_millis(200));
        });
    }

    #[test]
    fn send_request_retries_transient_network_error_then_succeeds() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Err(PlanradarApiError::new(
                    PlanradarApiErrorCode::Timeout,
                    None,
                    "Zeitüberschreitung",
                    "timeout",
                )),
                Ok(mock_response(200, r#"{"ok":true}"#)),
            ]);
            let client = PlanradarApiClient::with_transport_and_retry(
                Box::new(transport.clone()),
                RetryPolicy::immediate(3),
            );

            let response = client
                .send_request(PlanradarHttpMethod::Get, "/x", vec![], None, None)
                .await
                .expect("retried request should succeed after transient error");

            assert_eq!(response.status, 200);
            assert_eq!(transport.requests().len(), 2);
        });
    }

    #[test]
    fn send_request_gives_up_after_max_retries() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(503, "down")),
                Ok(mock_response(503, "down")),
            ]);
            let client = PlanradarApiClient::with_transport_and_retry(
                Box::new(transport.clone()),
                RetryPolicy::immediate(1),
            );

            let response = client
                .send_request(PlanradarHttpMethod::Get, "/x", vec![], None, None)
                .await
                .expect("should return the last response after exhausting retries");

            assert_eq!(response.status, 503);
            assert_eq!(transport.requests().len(), 2);
        });
    }

    #[test]
    fn send_request_does_not_retry_client_errors() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(404, "not found"))]);
            let client = PlanradarApiClient::with_transport_and_retry(
                Box::new(transport.clone()),
                RetryPolicy::immediate(3),
            );

            let response = client
                .send_request(PlanradarHttpMethod::Get, "/x", vec![], None, None)
                .await
                .expect("non-retryable response should return immediately");

            assert_eq!(response.status, 404);
            assert_eq!(transport.requests().len(), 1);
        });
    }
}
