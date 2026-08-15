use super::context::EventContext;
use super::events::TelemetryEvent;
use super::queue::DeliveryError;
#[cfg(test)]
use crate::integrations::http_record_replay::{RecordReplayConfig, RecordedRequest, VcrMode};
use serde_json::{json, Map, Value};
use tauri_plugin_http::reqwest;

pub const POSTHOG_ENDPOINT: &str = "https://eu.i.posthog.com/batch/";

/// Absent in development and CI builds, which makes the client inert there.
const BUILD_TIME_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");

pub struct PostHogClient {
    api_key: Option<String>,
    endpoint: String,
    http_client: reqwest::Client,
    #[cfg(test)]
    record_replay: Option<RecordReplayConfig>,
}

impl PostHogClient {
    pub fn from_build_config() -> Self {
        Self {
            api_key: BUILD_TIME_API_KEY.map(str::to_string),
            endpoint: POSTHOG_ENDPOINT.to_string(),
            http_client: reqwest::Client::new(),
            #[cfg(test)]
            record_replay: None,
        }
    }

    pub fn is_inert(&self) -> bool {
        self.api_key.is_none()
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

        #[cfg(test)]
        if let Some(record_replay) = &self.record_replay {
            if record_replay.mode() == VcrMode::Replay {
                let recorded = record_replay
                    .replay(&RecordedRequest {
                        method: "POST".to_string(),
                        path: "/batch/".to_string(),
                        query: Vec::new(),
                        body: Some(payload),
                    })
                    .map_err(|_| DeliveryError)?
                    .ok_or(DeliveryError)?;

                return (recorded.status < 400).then_some(()).ok_or(DeliveryError);
            }
        }

        let response = self
            .http_client
            .post(&self.endpoint)
            .header("content-type", "application/json")
            .body(payload.to_string())
            .send()
            .await
            .map_err(|_| DeliveryError)?;

        response
            .status()
            .is_success()
            .then_some(())
            .ok_or(DeliveryError)
    }

    #[cfg(test)]
    pub(crate) fn with_key_for_test(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_string()),
            endpoint: POSTHOG_ENDPOINT.to_string(),
            http_client: reqwest::Client::new(),
            record_replay: None,
        }
    }

    #[cfg(test)]
    fn with_replay_cassette(api_key: &str, cassette_file_name: &str) -> Self {
        Self {
            record_replay: Some(RecordReplayConfig::new(
                cassette_path_for_test(cassette_file_name),
                VcrMode::Replay,
            )),
            ..Self::with_key_for_test(api_key)
        }
    }
}

#[cfg(test)]
fn cassette_path_for_test(file_name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/cassettes")
        .join(file_name)
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
    use crate::integrations::telemetry::events::{Integration, Operation};

    fn context() -> EventContext {
        EventContext {
            install_id: "11111111-2222-4333-8444-555555555555".to_string(),
            app_version: "0.1.0".to_string(),
            os_name: "macos".to_string(),
            os_version: "15.3".to_string(),
        }
    }

    fn event() -> TelemetryEvent {
        TelemetryEvent::operation_completed(Operation::LoadWeekEvents, Integration::Zep, true, 42)
    }

    #[test]
    fn client_is_inert_without_a_build_time_key() {
        let client = PostHogClient::from_build_config();

        assert!(client.is_inert());
        assert_eq!(client.batch_payload(vec![event()], &context()), None);
    }

    #[test]
    fn batch_payload_carries_the_api_key_and_one_entry_per_event() {
        let client = PostHogClient::with_key_for_test("phc_test_key");

        let payload = client
            .batch_payload(vec![event(), event()], &context())
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
        let client = PostHogClient::with_key_for_test("phc_test_key");

        let payload = client
            .batch_payload(vec![event()], &context())
            .expect("a configured client builds a payload");

        assert_eq!(
            payload["batch"][0]["distinct_id"],
            "11111111-2222-4333-8444-555555555555"
        );
    }

    #[tokio::test]
    async fn capture_request_matches_the_recorded_batch_interaction() {
        let client = PostHogClient::with_replay_cassette("phc_test_key", "posthog-capture.json");

        client
            .send(vec![event()], &context())
            .await
            .expect("the recorded interaction should match the built request");
    }

    #[tokio::test]
    async fn a_rejected_batch_is_reported_as_a_delivery_error() {
        let client = PostHogClient::with_replay_cassette("phc_test_key", "posthog-capture.json");
        let mut rejected = context();
        rejected.install_id = "rejected-install-id".to_string();

        let result = client.send(vec![event()], &rejected).await;

        assert_eq!(result, Err(DeliveryError));
    }

    #[test]
    fn the_endpoint_is_the_eu_region_batch_url() {
        assert_eq!(POSTHOG_ENDPOINT, "https://eu.i.posthog.com/batch/");
    }
}
