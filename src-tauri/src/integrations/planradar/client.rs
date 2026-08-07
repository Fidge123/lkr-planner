use super::shared::{normalize_base_url, PlanradarApiError, PlanradarApiErrorCode};
#[cfg(test)]
use crate::integrations::http_record_replay::{
    RecordReplayConfig, RecordedInteraction, RecordedRequest, RecordedResponse, VcrMode,
};
use serde_json::Value;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tauri_plugin_http::reqwest;
use tokio::sync::Mutex;

const PLANRADAR_API_KEY_HEADER: &str = "X-PlanRadar-API-Key";

/// Planradar allows ~30 requests/minute; we stay well under because the same personal token may
/// be in use by other tools concurrently.
const PLANRADAR_RATE_LIMIT_MAX_REQUESTS: usize = 15;
const PLANRADAR_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
/// Planradar sends no `Retry-After`, so a 429 is assumed to cost at least a full window.
const PLANRADAR_RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(60);
/// Caps how long one request may wait for the budget, so a burst cannot hang a Tauri call.
const PLANRADAR_RATE_LIMIT_MAX_WAIT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
}

impl RetryPolicy {
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

#[derive(Default)]
struct RateLimiterState {
    timestamps: VecDeque<Instant>,
    blocked_until: Option<Instant>,
}

/// Sliding-window rate limiter. A `None` state disables it entirely.
#[derive(Clone)]
pub(super) struct RateLimiter {
    state: Option<Arc<Mutex<RateLimiterState>>>,
    max_requests: usize,
    window: Duration,
    cooldown: Duration,
    max_wait: Duration,
}

impl RateLimiter {
    /// Process-wide, because every command builds its own client but shares one account token.
    pub(super) fn global() -> Self {
        static GLOBAL: OnceLock<Arc<Mutex<RateLimiterState>>> = OnceLock::new();
        let state = GLOBAL
            .get_or_init(|| Arc::new(Mutex::new(RateLimiterState::default())))
            .clone();
        Self {
            state: Some(state),
            max_requests: PLANRADAR_RATE_LIMIT_MAX_REQUESTS,
            window: PLANRADAR_RATE_LIMIT_WINDOW,
            cooldown: PLANRADAR_RATE_LIMIT_COOLDOWN,
            max_wait: PLANRADAR_RATE_LIMIT_MAX_WAIT,
        }
    }

    #[cfg(test)]
    pub(super) fn disabled() -> Self {
        Self {
            state: None,
            max_requests: 0,
            window: Duration::ZERO,
            cooldown: Duration::ZERO,
            max_wait: Duration::ZERO,
        }
    }

    #[cfg(test)]
    fn for_test(
        max_requests: usize,
        window: Duration,
        cooldown: Duration,
        max_wait: Duration,
    ) -> Self {
        Self {
            state: Some(Arc::new(Mutex::new(RateLimiterState::default()))),
            max_requests,
            window,
            cooldown,
            max_wait,
        }
    }

    /// Computed once per `send_request` so its retries share one bound instead of each getting a
    /// fresh `max_wait`.
    fn deadline(&self) -> Option<Instant> {
        self.state.as_ref().map(|_| Instant::now() + self.max_wait)
    }

    /// Blocks until one more request fits the window and any 429 cooldown has elapsed, then
    /// records it. The lock is never held across the sleep, so concurrent callers re-evaluate.
    async fn acquire(&self, deadline: Option<Instant>) -> Result<(), PlanradarApiError> {
        let Some(state) = &self.state else {
            return Ok(());
        };

        loop {
            let wait = {
                let mut state = state.lock().await;
                let now = Instant::now();

                match state.blocked_until {
                    Some(until) if until > now => until.saturating_duration_since(now),
                    _ => {
                        state.blocked_until = None;
                        let wait = compute_wait(
                            &mut state.timestamps,
                            now,
                            self.max_requests,
                            self.window,
                        );
                        if wait.is_zero() {
                            state.timestamps.push_back(now);
                            return Ok(());
                        }
                        wait
                    }
                }
            };

            if let Some(deadline) = deadline {
                if Instant::now() + wait > deadline {
                    return Err(PlanradarApiError::new(
                        PlanradarApiErrorCode::RateLimited,
                        None,
                        "Planradar ist momentan überlastet. Bitte kurz warten und erneut versuchen.",
                        format!("Planradar rate-limit wait exceeded {:?}", self.max_wait),
                    ));
                }
            }
            tokio::time::sleep(wait).await;
        }
    }

    async fn record_rate_limit_hit(&self) {
        let Some(state) = &self.state else {
            return;
        };

        let mut state = state.lock().await;
        let until = Instant::now() + self.cooldown;
        // Extend, never shorten, an existing cooldown.
        state.blocked_until = Some(match state.blocked_until {
            Some(existing) if existing > until => existing,
            _ => until,
        });
    }
}

/// Evicts expired timestamps, then returns the wait before another request fits.
/// `Duration::ZERO` means a slot is free now and the caller must record the timestamp.
fn compute_wait(
    timestamps: &mut VecDeque<Instant>,
    now: Instant,
    max_requests: usize,
    window: Duration,
) -> Duration {
    while let Some(&oldest) = timestamps.front() {
        if now.duration_since(oldest) >= window {
            timestamps.pop_front();
        } else {
            break;
        }
    }

    if timestamps.len() < max_requests {
        return Duration::ZERO;
    }

    let oldest = *timestamps
        .front()
        .expect("a full window has at least one timestamp");
    (oldest + window).saturating_duration_since(now)
}

pub(super) struct PlanradarApiClient {
    transport: Box<dyn PlanradarHttpTransport>,
    retry: RetryPolicy,
    rate_limiter: RateLimiter,
}

impl PlanradarApiClient {
    pub(super) fn new(base_url: &str) -> Result<Self, PlanradarApiError> {
        let transport = ReqwestTransport::new(base_url)?;
        Ok(Self {
            transport: Box::new(transport),
            retry: RetryPolicy::standard(),
            rate_limiter: RateLimiter::global(),
        })
    }

    #[cfg(test)]
    pub(super) fn with_transport(transport: Box<dyn PlanradarHttpTransport>) -> Self {
        Self {
            transport,
            retry: RetryPolicy::none(),
            rate_limiter: RateLimiter::disabled(),
        }
    }

    #[cfg(test)]
    pub(super) fn with_transport_and_retry(
        transport: Box<dyn PlanradarHttpTransport>,
        retry: RetryPolicy,
    ) -> Self {
        Self {
            transport,
            retry,
            rate_limiter: RateLimiter::disabled(),
        }
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
            rate_limiter: RateLimiter::disabled(),
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
            rate_limiter: RateLimiter::global(),
        })
    }

    pub(super) async fn send_request(
        &self,
        method: PlanradarHttpMethod,
        path: &str,
        query: Vec<(String, String)>,
        body: Option<Value>,
        api_key: Option<String>,
    ) -> Result<PlanradarHttpResponse, PlanradarApiError> {
        let deadline = self.rate_limiter.deadline();
        let mut attempt = 0;
        loop {
            let request = PlanradarHttpRequest {
                method,
                path: path.to_string(),
                query: query.clone(),
                body: body.clone(),
                api_key: api_key.clone(),
            };

            // Inside the loop: retries are real requests and must count against the budget.
            self.rate_limiter.acquire(deadline).await?;

            let result = self.transport.send(request).await;

            if matches!(&result, Ok(response) if response.status == 429) {
                self.rate_limiter.record_rate_limit_hit().await;
            }

            let should_retry = attempt < self.retry.max_retries
                && is_idempotent(method)
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

/// A retried POST whose first attempt succeeded server-side but lost its response would create a
/// duplicate project, so POSTs are never retried.
fn is_idempotent(method: PlanradarHttpMethod) -> bool {
    matches!(method, PlanradarHttpMethod::Get | PlanradarHttpMethod::Put)
}

/// 429 is excluded on purpose: retrying only deepens Planradar's forced cooldown.
fn is_retryable_status(status: u16) -> bool {
    (500..=599).contains(&status)
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
pub(super) fn cassette_path_for_test(file_name: &str) -> std::path::PathBuf {
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
    fn send_request_does_not_retry_rate_limit() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(429, "rate limited"))]);
            let client = PlanradarApiClient::with_transport_and_retry(
                Box::new(transport.clone()),
                RetryPolicy::immediate(3),
            );

            let response = client
                .send_request(PlanradarHttpMethod::Get, "/x", vec![], None, None)
                .await
                .expect("429 should return without retrying");

            assert_eq!(response.status, 429);
            assert_eq!(transport.requests().len(), 1);
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
    fn compute_wait_allows_requests_under_the_limit() {
        let now = Instant::now();
        let mut timestamps = VecDeque::from(vec![now - Duration::from_secs(10)]);

        let wait = compute_wait(&mut timestamps, now, 15, Duration::from_secs(60));

        assert_eq!(wait, Duration::ZERO);
    }

    #[test]
    fn compute_wait_evicts_expired_timestamps() {
        let now = Instant::now();
        let mut timestamps = VecDeque::from(vec![
            now - Duration::from_secs(120),
            now - Duration::from_secs(90),
            now - Duration::from_secs(5),
        ]);

        let wait = compute_wait(&mut timestamps, now, 2, Duration::from_secs(60));

        assert_eq!(wait, Duration::ZERO);
        assert_eq!(timestamps.len(), 1, "expired timestamps should be evicted");
    }

    #[test]
    fn compute_wait_blocks_when_window_is_full() {
        let now = Instant::now();
        let mut timestamps = VecDeque::from(vec![
            now - Duration::from_secs(50),
            now - Duration::from_secs(20),
        ]);

        let wait = compute_wait(&mut timestamps, now, 2, Duration::from_secs(60));

        assert!(
            wait > Duration::from_secs(9) && wait <= Duration::from_secs(10),
            "expected ~10s wait, got {wait:?}"
        );
    }

    #[test]
    fn rate_limiter_throttles_once_the_window_is_full() {
        tauri::async_runtime::block_on(async {
            // Only the lower bound is asserted: a sleep can overshoot under load but never
            // finish early, so this stays deterministic on a busy CI machine.
            let limiter = RateLimiter::for_test(
                2,
                Duration::from_millis(80),
                Duration::from_millis(80),
                Duration::from_secs(10),
            );

            limiter
                .acquire(limiter.deadline())
                .await
                .expect("first acquire should pass");
            limiter
                .acquire(limiter.deadline())
                .await
                .expect("second acquire should pass");

            let started_at = Instant::now();
            limiter
                .acquire(limiter.deadline())
                .await
                .expect("third acquire should pass");
            assert!(
                started_at.elapsed() >= Duration::from_millis(70),
                "third acquire should block until the window slides"
            );
        });
    }

    #[test]
    fn rate_limiter_holds_back_during_429_cooldown() {
        tauri::async_runtime::block_on(async {
            // Generous window so only the cooldown gates the next acquire.
            let limiter = RateLimiter::for_test(
                100,
                Duration::from_secs(10),
                Duration::from_millis(80),
                Duration::from_secs(10),
            );

            limiter.record_rate_limit_hit().await;

            let started_at = Instant::now();
            limiter
                .acquire(limiter.deadline())
                .await
                .expect("acquire should pass once cooldown elapses");
            assert!(
                started_at.elapsed() >= Duration::from_millis(70),
                "acquire should wait out the 429 cooldown"
            );
        });
    }

    #[test]
    fn rate_limiter_gives_up_after_max_wait() {
        tauri::async_runtime::block_on(async {
            let limiter = RateLimiter::for_test(
                1,
                Duration::from_secs(30),
                Duration::from_secs(30),
                Duration::from_millis(20),
            );

            limiter
                .acquire(limiter.deadline())
                .await
                .expect("first acquire should pass");
            let error = limiter
                .acquire(limiter.deadline())
                .await
                .expect_err("second acquire should give up after max wait");
            assert_eq!(error.code, PlanradarApiErrorCode::RateLimited);
        });
    }

    #[test]
    fn rate_limit_deadline_spans_retries_of_one_request() {
        tauri::async_runtime::block_on(async {
            let limiter = RateLimiter::for_test(
                1,
                Duration::from_secs(30),
                Duration::from_secs(30),
                Duration::from_millis(20),
            );

            let deadline = limiter.deadline();
            limiter
                .acquire(deadline)
                .await
                .expect("first acquire should pass");

            let error = limiter
                .acquire(deadline)
                .await
                .expect_err("retry sharing the deadline should give up");
            assert_eq!(error.code, PlanradarApiErrorCode::RateLimited);
        });
    }

    #[test]
    fn disabled_rate_limiter_never_blocks() {
        tauri::async_runtime::block_on(async {
            let limiter = RateLimiter::disabled();
            let started_at = Instant::now();
            for _ in 0..100 {
                limiter
                    .acquire(limiter.deadline())
                    .await
                    .expect("disabled limiter never fails");
            }
            assert!(started_at.elapsed() < Duration::from_secs(1));
        });
    }

    #[test]
    fn send_request_does_not_retry_non_idempotent_post() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(503, "down"))]);
            let client = PlanradarApiClient::with_transport_and_retry(
                Box::new(transport.clone()),
                RetryPolicy::immediate(3),
            );

            let response = client
                .send_request(PlanradarHttpMethod::Post, "/x", vec![], None, None)
                .await
                .expect("post should return the first response without retrying");

            assert_eq!(response.status, 503);
            assert_eq!(transport.requests().len(), 1);
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
