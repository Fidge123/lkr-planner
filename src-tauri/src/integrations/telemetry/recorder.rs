use super::events::TelemetryEvent;
use crate::integrations::local_store::types::TelemetrySettings;
use std::sync::Mutex;
use tokio::sync::mpsc;

const COMMAND_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug)]
pub enum TelemetryCommand {
    Record(Box<TelemetryEvent>),
    DiscardPending,
    Flush,
}

#[derive(Debug, Default)]
struct ConsentState {
    enabled: bool,
    install_id: Option<String>,
}

/// Consent is cached in memory; the local store is only read at startup and
/// written when the user changes the setting.
pub struct TelemetryRecorder {
    state: Mutex<ConsentState>,
    sender: mpsc::Sender<TelemetryCommand>,
}

impl TelemetryRecorder {
    pub fn new(sender: mpsc::Sender<TelemetryCommand>) -> Self {
        Self {
            state: Mutex::new(ConsentState::default()),
            sender,
        }
    }

    pub fn channel() -> (Self, mpsc::Receiver<TelemetryCommand>) {
        let (sender, receiver) = mpsc::channel(COMMAND_CHANNEL_CAPACITY);
        (Self::new(sender), receiver)
    }

    pub fn apply_settings(&self, settings: &TelemetrySettings) {
        let mut state = self.lock();
        state.enabled = settings.enabled;
        if settings.install_id.is_some() {
            state.install_id = settings.install_id.clone();
        }
    }

    pub fn set_enabled(&self, enabled: bool) -> TelemetrySettings {
        let settings = {
            let mut state = self.lock();
            state.enabled = enabled;
            if enabled && state.install_id.is_none() {
                state.install_id = Some(generate_install_id());
            }

            TelemetrySettings {
                enabled: state.enabled,
                install_id: state.install_id.clone(),
            }
        };

        if !enabled {
            self.send(TelemetryCommand::DiscardPending);
        }

        settings
    }

    pub fn is_enabled(&self) -> bool {
        self.lock().enabled
    }

    pub fn install_id(&self) -> Option<String> {
        self.lock().install_id.clone()
    }

    pub fn record(&self, event: TelemetryEvent) {
        if !self.ensure_active() {
            return;
        }

        self.send(TelemetryCommand::Record(Box::new(event)));
    }

    pub fn request_flush(&self) {
        if self.is_enabled() {
            self.send(TelemetryCommand::Flush);
        }
    }

    /// A store enabled by an older build can lack an identifier; mint one rather
    /// than dropping the event.
    fn ensure_active(&self) -> bool {
        let mut state = self.lock();
        if !state.enabled {
            return false;
        }

        if state.install_id.is_none() {
            state.install_id = Some(generate_install_id());
        }

        true
    }

    /// A full channel drops the event: telemetry never applies back-pressure to a
    /// user-facing operation.
    fn send(&self, command: TelemetryCommand) {
        let _ = self.sender.try_send(command);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ConsentState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn generate_install_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::telemetry::events::{Integration, Operation};

    impl TelemetryRecorder {
        fn for_test() -> (Self, mpsc::Receiver<TelemetryCommand>) {
            Self::channel()
        }
    }

    fn test_event() -> TelemetryEvent {
        TelemetryEvent::operation_completed(Operation::LoadWeekEvents, Integration::Zep, true, 42)
    }

    fn enabled_settings() -> TelemetrySettings {
        TelemetrySettings {
            enabled: true,
            install_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        }
    }

    #[test]
    fn records_nothing_while_telemetry_is_disabled() {
        let (recorder, mut receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&TelemetrySettings::default());

        recorder.record(test_event());

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn records_events_once_telemetry_is_enabled() {
        let (recorder, mut receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&enabled_settings());

        recorder.record(test_event());

        assert!(matches!(
            receiver.try_recv(),
            Ok(TelemetryCommand::Record(_))
        ));
    }

    #[test]
    fn disabling_discards_pending_events() {
        let (recorder, mut receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&enabled_settings());
        recorder.record(test_event());

        recorder.set_enabled(false);

        let mut commands = Vec::new();
        while let Ok(command) = receiver.try_recv() {
            commands.push(command);
        }

        assert!(matches!(
            commands.last(),
            Some(TelemetryCommand::DiscardPending)
        ));
        assert!(!recorder.is_enabled());
    }

    #[test]
    fn records_nothing_after_being_disabled() {
        let (recorder, mut receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&enabled_settings());
        recorder.set_enabled(false);
        while receiver.try_recv().is_ok() {}

        recorder.record(test_event());

        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn install_id_is_generated_on_first_activation() {
        let (recorder, _receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&TelemetrySettings::default());

        let settings = recorder.set_enabled(true);

        let install_id = settings.install_id.expect("activation should mint an id");
        assert_eq!(install_id.len(), 36);
        assert_eq!(
            uuid::Uuid::parse_str(&install_id)
                .unwrap()
                .get_version_num(),
            4
        );
        assert_eq!(recorder.install_id(), Some(install_id));
    }

    #[test]
    fn install_id_is_stable_across_reloads() {
        let (recorder, _receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&enabled_settings());
        let first = recorder.install_id();

        recorder.apply_settings(&enabled_settings());

        assert_eq!(recorder.install_id(), first);
        assert_eq!(
            first.as_deref(),
            Some("11111111-2222-4333-8444-555555555555")
        );
    }

    #[test]
    fn reactivation_keeps_the_existing_install_id() {
        let (recorder, _receiver) = TelemetryRecorder::for_test();
        recorder.apply_settings(&enabled_settings());

        recorder.set_enabled(false);
        let settings = recorder.set_enabled(true);

        assert_eq!(
            settings.install_id.as_deref(),
            Some("11111111-2222-4333-8444-555555555555")
        );
    }

    #[test]
    fn install_id_is_not_derived_from_user_or_host_data() {
        let (first_recorder, _first) = TelemetryRecorder::for_test();
        let (second_recorder, _second) = TelemetryRecorder::for_test();
        first_recorder.apply_settings(&TelemetrySettings::default());
        second_recorder.apply_settings(&TelemetrySettings::default());

        let first = first_recorder.set_enabled(true).install_id;
        let second = second_recorder.set_enabled(true).install_id;

        assert_ne!(first, second);
    }
}
