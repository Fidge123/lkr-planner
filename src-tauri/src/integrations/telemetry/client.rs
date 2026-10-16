use super::context::EventContext;
use super::events::TelemetryEvent;
use super::queue::DeliveryError;
use serde_json::{json, Map, Value};
use tauri_plugin_http::reqwest;

pub const POSTHOG_ENDPOINT: &str = "https://eu.i.posthog.com/batch/";

/// Absent in development and CI builds, which makes the client inert there.
const BUILD_TIME_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");

pub struct PostHogClient {
    api_key: Option<String>,
    endpoint: String,
    http_client: reqwest::Client,
}

impl PostHogClient {
    pub fn from_build_config() -> Self {
        Self::with_api_key(BUILD_TIME_API_KEY)
    }

    pub(crate) fn with_api_key(api_key: Option<&str>) -> Self {
        Self {
            api_key: api_key.map(str::to_string),
            endpoint: POSTHOG_ENDPOINT.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn batch_payload(
        &self,
        events: Vec<TelemetryEvent>,
        context: &EventContext,
    ) -> Option<Value> {
        let api_key = self.api_key.as_ref()?;
        if events.is_empty() {
            return None;
        }

        let batch: Vec<Value> = events
            .into_iter()
            .map(|event| {
                json!({
                    "event": event.name(),
                    "distinct_id": context.install_id,
                    "properties": enrich(event.properties(), context),
                })
            })
            .collect();

        Some(json!({ "api_key": api_key, "batch": batch }))
    }

    pub async fn send(
        &self,
        events: Vec<TelemetryEvent>,
        context: &EventContext,
    ) -> Result<(), DeliveryError> {
        let Some(payload) = self.batch_payload(events, context) else {
            return Ok(());
        };

        let response = self
            .http_client
            .post(&self.endpoint)
            .header("content-type", "application/json")
            .body(payload.to_string())
            .send()
            .await
            .map_err(|_| DeliveryError)?;

        delivery_result(response.status())
    }
}

fn delivery_result(status: reqwest::StatusCode) -> Result<(), DeliveryError> {
    status.is_success().then_some(()).ok_or(DeliveryError)
}

fn enrich(properties: &Map<String, Value>, context: &EventContext) -> Map<String, Value> {
    let mut enriched = properties.clone();
    enriched.insert(
        "app_version".to_string(),
        Value::String(context.app_version.clone()),
    );
    enriched.insert(
        "os_name".to_string(),
        Value::String(context.os_name.clone()),
    );
    enriched.insert(
        "os_version".to_string(),
        Value::String(context.os_version.clone()),
    );

    enriched
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::telemetry::test_support::{event, INSTALL_ID};

    fn context() -> EventContext {
        EventContext {
            install_id: INSTALL_ID.to_string(),
            app_version: "0.1.0".to_string(),
            os_name: "macos".to_string(),
            os_version: "15.3".to_string(),
        }
    }

    #[test]
    fn client_is_inert_without_an_api_key() {
        let client = PostHogClient::with_api_key(None);

        assert_eq!(client.batch_payload(vec![event(42)], &context()), None);
    }

    #[test]
    fn batch_payload_carries_the_api_key_and_one_entry_per_event() {
        let client = PostHogClient::with_api_key(Some("phc_test_key"));

        let payload = client
            .batch_payload(vec![event(42), event(42)], &context())
            .expect("a configured client builds a payload");

        assert_eq!(
            payload["api_key"],
            Value::String("phc_test_key".to_string())
        );
        assert_eq!(payload["batch"].as_array().unwrap().len(), 2);
        assert_eq!(payload["batch"][0]["event"], "operation_completed");
    }

    #[test]
    fn every_entry_is_identified_by_the_install_id() {
        let client = PostHogClient::with_api_key(Some("phc_test_key"));

        let payload = client
            .batch_payload(vec![event(42)], &context())
            .expect("a configured client builds a payload");

        assert_eq!(payload["batch"][0]["distinct_id"], INSTALL_ID);
    }

    #[test]
    fn only_a_success_status_counts_as_delivered() {
        assert_eq!(delivery_result(reqwest::StatusCode::OK), Ok(()));
        assert_eq!(
            delivery_result(reqwest::StatusCode::UNAUTHORIZED),
            Err(DeliveryError)
        );
        assert_eq!(
            delivery_result(reqwest::StatusCode::SERVICE_UNAVAILABLE),
            Err(DeliveryError)
        );
    }

    #[tokio::test]
    #[cfg_attr(not(posthog_api_key), ignore = "POSTHOG_API_KEY not set")]
    async fn delivers_a_batch_to_the_live_posthog_api() {
        const LIVE_TEST_INSTALL_ID: &str = "lkr-planner-integration-test";
        let client = PostHogClient::from_build_config();
        let context = EventContext::current(LIVE_TEST_INSTALL_ID.to_string());
        assert!(
            client.batch_payload(vec![event(42)], &context).is_some(),
            "the build carries no PostHog key, so nothing would be sent"
        );

        let result = client.send(vec![event(42)], &context).await;

        assert_eq!(
            result,
            Ok(()),
            "PostHog was unreachable or rejected the batch"
        );
    }

    #[test]
    fn the_endpoint_is_the_eu_region_batch_url() {
        assert_eq!(POSTHOG_ENDPOINT, "https://eu.i.posthog.com/batch/");
    }
}
