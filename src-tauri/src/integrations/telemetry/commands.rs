use super::events::{ErrorCode, Integration, Operation, TelemetryEvent};
use super::state::TelemetryState;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FrontendErrorSource {
    Render,
    UncaughtError,
    UnhandledRejection,
}

impl FrontendErrorSource {
    fn operation(self) -> Operation {
        match self {
            Self::Render => Operation::FrontendRender,
            Self::UncaughtError => Operation::FrontendUncaughtError,
            Self::UnhandledRejection => Operation::FrontendPromiseRejection,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FrontendErrorInput {
    pub source: FrontendErrorSource,
    pub name: String,
    pub message: String,
    pub context: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn telemetry_capture_frontend_error(app: tauri::AppHandle, error: FrontendErrorInput) {
    let event = TelemetryEvent::error_occurred(
        error.source.operation(),
        Integration::Frontend,
        ErrorCode::from_untrusted(&error.name),
        Some(&error.message),
    )
    .with_context(error.context.as_deref());

    app.state::<TelemetryState>().record(event);
}
