use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContext {
    pub install_id: String,
    pub app_version: String,
    pub os_name: String,
    pub os_version: String,
}

impl EventContext {
    pub fn current(install_id: String) -> Self {
        Self {
            install_id,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_name: std::env::consts::OS.to_string(),
            os_version: os_version().to_string(),
        }
    }
}

/// Resolving the version can shell out on some platforms, so it is read once.
fn os_version() -> &'static str {
    static OS_VERSION: OnceLock<String> = OnceLock::new();
    OS_VERSION.get_or_init(|| os_info::get().version().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::telemetry::client::PostHogClient;
    use crate::integrations::telemetry::events::{Integration, Operation, TelemetryEvent};
    use crate::integrations::telemetry::test_support::INSTALL_ID;

    #[test]
    fn context_reports_the_app_version_from_the_crate_metadata() {
        let context = EventContext::current(INSTALL_ID.to_string());

        assert_eq!(context.app_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn context_reports_a_non_empty_operating_system_name_and_version() {
        let context = EventContext::current(INSTALL_ID.to_string());

        assert!(!context.os_name.is_empty());
        assert!(!context.os_version.is_empty());
    }

    #[test]
    fn every_transmitted_event_carries_the_full_context() {
        let client = PostHogClient::with_key_for_test("phc_test_key");
        let context = EventContext::current(INSTALL_ID.to_string());
        let events = vec![
            TelemetryEvent::operation_completed(
                Operation::LoadWeekEvents,
                Integration::Zep,
                true,
                42,
            ),
            TelemetryEvent::app_started(1200),
        ];

        let payload = client
            .batch_payload(events, &context)
            .expect("a configured client builds a payload");

        for entry in payload["batch"].as_array().expect("batch is an array") {
            assert_eq!(entry["distinct_id"], context.install_id);
            assert_eq!(entry["properties"]["app_version"], context.app_version);
            assert_eq!(entry["properties"]["os_name"], context.os_name);
            assert_eq!(entry["properties"]["os_version"], context.os_version);
        }
    }

    #[test]
    fn events_from_two_releases_stay_distinguishable() {
        let client = PostHogClient::with_key_for_test("phc_test_key");
        let mut previous = EventContext::current("install".to_string());
        previous.app_version = "0.0.9".to_string();
        let current = EventContext::current("install".to_string());

        let previous_payload = client
            .batch_payload(vec![TelemetryEvent::app_started(1)], &previous)
            .unwrap();
        let current_payload = client
            .batch_payload(vec![TelemetryEvent::app_started(1)], &current)
            .unwrap();

        assert_ne!(
            previous_payload["batch"][0]["properties"]["app_version"],
            current_payload["batch"][0]["properties"]["app_version"]
        );
    }
}
