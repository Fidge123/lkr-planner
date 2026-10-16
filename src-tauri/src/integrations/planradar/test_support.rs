use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::client::{
    BoxFuture, PlanradarApiClient, PlanradarHttpRequest, PlanradarHttpResponse,
    PlanradarHttpTransport, RetryPolicy,
};
use super::shared::PlanradarApiError;

#[derive(Clone)]
pub(super) struct MockTransport {
    responses: Arc<Mutex<VecDeque<Result<PlanradarHttpResponse, PlanradarApiError>>>>,
    requests: Arc<Mutex<Vec<PlanradarHttpRequest>>>,
}

impl MockTransport {
    pub(super) fn new(responses: Vec<Result<PlanradarHttpResponse, PlanradarApiError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(super) fn requests(&self) -> Vec<PlanradarHttpRequest> {
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

pub(super) fn mock_client(
    responses: Vec<Result<PlanradarHttpResponse, PlanradarApiError>>,
) -> (PlanradarApiClient, MockTransport) {
    let transport = MockTransport::new(responses);
    (
        PlanradarApiClient::with_transport(Box::new(transport.clone())),
        transport,
    )
}

pub(super) fn mock_client_with_retry(
    responses: Vec<Result<PlanradarHttpResponse, PlanradarApiError>>,
    retry: RetryPolicy,
) -> (PlanradarApiClient, MockTransport) {
    let transport = MockTransport::new(responses);
    (
        PlanradarApiClient::with_transport_and_retry(Box::new(transport.clone()), retry),
        transport,
    )
}

pub(super) fn mock_response(status: u16, body: &str) -> PlanradarHttpResponse {
    PlanradarHttpResponse {
        status,
        body: body.to_string(),
    }
}
