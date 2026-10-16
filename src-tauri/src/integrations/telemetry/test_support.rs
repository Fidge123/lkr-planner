use super::events::{Integration, Operation, TelemetryEvent};
use super::recorder::{TelemetryCommand, TelemetryRecorder};
use crate::integrations::local_store::types::TelemetrySettings;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

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

pub(crate) fn enabled_recorder() -> (Arc<TelemetryRecorder>, Receiver<TelemetryCommand>) {
    let (recorder, receiver) = TelemetryRecorder::channel();
    recorder.apply_settings(&enabled());
    (Arc::new(recorder), receiver)
}

pub(crate) fn recorded(receiver: &mut Receiver<TelemetryCommand>) -> Vec<TelemetryEvent> {
    let mut events = Vec::new();
    while let Ok(command) = receiver.try_recv() {
        if let TelemetryCommand::Record(event) = command {
            events.push(*event);
        }
    }

    events
}
