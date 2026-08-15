use super::events::{ErrorCode, Integration, Operation, TelemetryEvent};
use super::state::TelemetryState;
use crate::integrations::local_store::types::{StoreError, TelemetrySettings};
use crate::integrations::local_store::{load_local_store, save_local_store};
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
pub fn telemetry_get_settings(app: tauri::AppHandle) -> Result<TelemetrySettings, StoreError> {
    Ok(load_local_store(app)?.telemetry)
}

#[tauri::command]
#[specta::specta]
pub fn telemetry_set_enabled(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<TelemetrySettings, StoreError> {
    let settings = app
        .state::<TelemetryState>()
        .recorder()
        .set_enabled(enabled);

    let mut store = load_local_store(app.clone())?;
    store.telemetry = settings.clone();
    save_local_store(app, store)?;

    Ok(settings)
}

#[tauri::command]
#[specta::specta]
pub fn telemetry_capture_frontend_error(app: tauri::AppHandle, error: FrontendErrorInput) {
    let event = TelemetryEvent::error_occurred(
        error.source.operation(),
        Integration::Frontend,
        ErrorCode::from_untrusted(&error.name),
        &error.message,
    )
    .with_context(error.context.as_deref());

    app.state::<TelemetryState>().record(event);
}
