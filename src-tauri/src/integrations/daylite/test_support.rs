use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::client::{BoxFuture, DayliteHttpRequest, DayliteHttpResponse, DayliteHttpTransport};
use super::shared::{DayliteApiError, DayliteTokenState};

#[derive(Clone)]
pub(super) struct MockTransport {
    responses: Arc<Mutex<VecDeque<Result<DayliteHttpResponse, DayliteApiError>>>>,
    requests: Arc<Mutex<Vec<DayliteHttpRequest>>>,
}

impl MockTransport {
    pub(super) fn new(responses: Vec<Result<DayliteHttpResponse, DayliteApiError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(super) fn requests(&self) -> Vec<DayliteHttpRequest> {
        self.requests
            .lock()
            .expect("request lock should succeed")
            .clone()
    }
}

impl DayliteHttpTransport for MockTransport {
    fn send<'a>(
        &'a self,
        request: DayliteHttpRequest,
    ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>> {
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

pub(super) fn mock_response(status: u16, body: &str) -> DayliteHttpResponse {
    DayliteHttpResponse {
        status,
        body: body.to_string(),
    }
}

pub(super) fn token_state(access_token: &str, refresh_token: &str) -> DayliteTokenState {
    DayliteTokenState {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
        access_token_expires_at_ms: Some(u64::MAX),
    }
}

pub(super) fn valid_token_state() -> DayliteTokenState {
    token_state("at", "rt")
}
