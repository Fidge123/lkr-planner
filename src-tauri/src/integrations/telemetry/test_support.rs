use super::events::{Integration, Operation, TelemetryEvent};
use crate::integrations::local_store::types::TelemetrySettings;

pub(super) const INSTALL_ID: &str = "11111111-2222-4333-8444-555555555555";

pub(super) fn enabled() -> TelemetrySettings {
    TelemetrySettings {
        enabled: true,
        install_id: Some(INSTALL_ID.to_string()),
    }
}

pub(super) fn event(duration_ms: u64) -> TelemetryEvent {
    TelemetryEvent::operation_completed(
        Operation::LoadWeekEvents,
        Integration::Zep,
        true,
        duration_ms,
    )
}
