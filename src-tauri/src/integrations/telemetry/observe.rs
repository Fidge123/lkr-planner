use super::events::{ErrorCode, Integration, Operation, TelemetryEvent};
use super::recorder::TelemetryRecorder;
use super::state::TelemetryState;
use crate::integrations::daylite::shared::DayliteApiError;
use crate::integrations::local_store::types::StoreError;
use crate::integrations::zep::types::ZepError;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use tauri::Manager;

/// The closed set of dimensions an error contributes to an event.
pub trait TelemetryError {
    fn error_code(&self) -> ErrorCode;
    fn technical_message(&self) -> Option<&str>;
    fn http_status(&self) -> Option<u16> {
        None
    }
}

impl TelemetryError for StoreError {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::from_enum(&self.code)
    }

    fn technical_message(&self) -> Option<&str> {
        Some(&self.technical_message)
    }
}

impl TelemetryError for ZepError {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::from_enum(&self.code)
    }

    fn technical_message(&self) -> Option<&str> {
        Some(&self.technical_message)
    }
}

impl TelemetryError for DayliteApiError {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::from_enum(&self.code)
    }

    fn technical_message(&self) -> Option<&str> {
        Some(&self.technical_message)
    }

    fn http_status(&self) -> Option<u16> {
        self.http_status
    }
}

/// The string is the German user message and can name a project, so it is not sent.
impl TelemetryError for String {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::from_enum(&"COMMAND_FAILED")
    }

    fn technical_message(&self) -> Option<&str> {
        None
    }
}

pub async fn observe<T, E, F>(
    app: &tauri::AppHandle,
    operation: Operation,
    integration: Integration,
    future: F,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
    E: TelemetryError,
{
    let recorder = app
        .try_state::<TelemetryState>()
        .map(|state| Arc::clone(state.recorder()));

    observe_with(recorder.as_deref(), operation, integration, future).await
}

/// A failure at a call site that has no structured error type of its own.
pub struct RequestFailure {
    pub code: &'static str,
    pub message: Option<String>,
    pub http_status: Option<u16>,
}

impl RequestFailure {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: Some(message.into()),
            http_status: None,
        }
    }

    pub fn with_status(code: &'static str, http_status: u16) -> Self {
        Self {
            code,
            message: None,
            http_status: Some(http_status),
        }
    }
}

impl TelemetryError for RequestFailure {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::from_enum(&self.code)
    }

    fn technical_message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn http_status(&self) -> Option<u16> {
        self.http_status
    }
}

/// Failure only: the command wrapping this site already reports a duration.
pub fn record_failure<E: TelemetryError>(
    app: &tauri::AppHandle,
    operation: Operation,
    integration: Integration,
    error: &E,
) {
    let Some(state) = app.try_state::<TelemetryState>() else {
        return;
    };

    state.record(
        TelemetryEvent::error_occurred(
            operation,
            integration,
            error.error_code(),
            error.technical_message(),
        )
        .with_http_status(error.http_status()),
    );
}

pub async fn observe_with<T, E, F>(
    recorder: Option<&TelemetryRecorder>,
    operation: Operation,
    integration: Integration,
    future: F,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
    E: TelemetryError,
{
    let started_at = Instant::now();
    let result = future.await;

    let Some(recorder) = recorder else {
        return result;
    };

    let duration_ms = started_at.elapsed().as_millis() as u64;
    let http_status = result.as_ref().err().and_then(TelemetryError::http_status);

    recorder.record(
        TelemetryEvent::operation_completed(operation, integration, result.is_ok(), duration_ms)
            .with_http_status(http_status),
    );

    if let Err(error) = &result {
        recorder.record(
            TelemetryEvent::error_occurred(
                operation,
                integration,
                error.error_code(),
                error.technical_message(),
            )
            .with_http_status(http_status),
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::local_store::types::{StoreError, StoreErrorCode, TelemetrySettings};
    use crate::integrations::telemetry::recorder::{TelemetryCommand, TelemetryRecorder};

    fn enabled() -> TelemetrySettings {
        TelemetrySettings {
            enabled: true,
            install_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        }
    }

    fn recorded(
        receiver: &mut tokio::sync::mpsc::Receiver<TelemetryCommand>,
    ) -> Vec<TelemetryEvent> {
        let mut events = Vec::new();
        while let Ok(command) = receiver.try_recv() {
            if let TelemetryCommand::Record(event) = command {
                events.push(*event);
            }
        }

        events
    }

    #[tokio::test]
    async fn a_successful_operation_emits_one_completion_event() {
        let (recorder, mut receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());

        let result: Result<u8, StoreError> = observe_with(
            Some(&recorder),
            Operation::LoadLocalStore,
            Integration::LocalStore,
            async { Ok(7) },
        )
        .await;

        assert_eq!(result.unwrap(), 7);
        let events = recorded(&mut receiver);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name(), "operation_completed");
        assert_eq!(events[0].properties()["success"], true);
        assert!(events[0].properties().contains_key("duration_ms"));
    }

    #[tokio::test]
    async fn a_failed_operation_emits_a_completion_and_an_error_event() {
        let (recorder, mut receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());

        let result: Result<u8, StoreError> = observe_with(
            Some(&recorder),
            Operation::SaveLocalStore,
            Integration::LocalStore,
            async {
                Err(StoreError {
                    code: StoreErrorCode::WriteFailed,
                    user_message: "Die lokale Konfiguration konnte nicht gespeichert werden."
                        .to_string(),
                    technical_message:
                        "Datei konnte nicht geschrieben werden (/Users/flori/store.json)"
                            .to_string(),
                })
            },
        )
        .await;

        assert!(result.is_err());
        let events = recorded(&mut receiver);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].properties()["success"], false);
        assert!(events[0].properties().contains_key("duration_ms"));
        assert_eq!(events[1].name(), "error_occurred");
        assert_eq!(events[1].properties()["code"], "WRITE_FAILED");
        assert!(!events[1].properties()["message"]
            .as_str()
            .unwrap()
            .contains("flori"));
    }

    #[tokio::test]
    async fn a_string_error_carries_no_message_at_all() {
        let (recorder, mut receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());

        let result: Result<u8, String> = observe_with(
            Some(&recorder),
            Operation::LoadWeekEvents,
            Integration::Zep,
            async { Err("Projekt Musterbau konnte nicht geladen werden".to_string()) },
        )
        .await;

        assert!(result.is_err());
        let events = recorded(&mut receiver);
        assert_eq!(events[1].name(), "error_occurred");
        assert!(!events[1].properties().contains_key("message"));
        assert_eq!(events[1].properties()["code"], "COMMAND_FAILED");
    }

    #[tokio::test]
    async fn without_a_recorder_the_result_passes_through_untouched() {
        let result: Result<u8, StoreError> = observe_with(
            None,
            Operation::LoadLocalStore,
            Integration::LocalStore,
            async { Ok(3) },
        )
        .await;

        assert_eq!(result.unwrap(), 3);
    }
}
