use crate::error::AppError;
use crate::integrations::local_store::LocalStore;
use crate::integrations::telemetry::events::{Integration, Operation, TelemetryEvent};
use crate::integrations::telemetry::observe::{RequestFailure, TelemetryError};
use crate::integrations::telemetry::recorder::TelemetryRecorder;
use crate::integrations::telemetry::state::TelemetryState;
use crate::integrations::zep::{load_zep_credentials, ZepStoredCredentials};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_http::reqwest;

pub(crate) struct CaldavSession {
    pub(crate) client: reqwest::Client,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) base_url: String,
    pub(crate) absence_urls: Vec<String>,
    pub(crate) telemetry: Option<Arc<TelemetryRecorder>>,
}

impl CaldavSession {
    pub(crate) fn record_failure(&self, operation: Operation, failure: RequestFailure) {
        let Some(recorder) = &self.telemetry else {
            return;
        };

        recorder.record(
            TelemetryEvent::error_occurred(
                operation,
                Integration::Zep,
                failure.error_code(),
                failure.technical_message(),
            )
            .with_http_status(failure.http_status()),
        );
    }
}

pub(super) fn resolve_employee_calendar_url(
    store: &LocalStore,
    employee_reference: &str,
) -> Result<String, String> {
    store
        .employee_settings
        .iter()
        .find(|s| s.daylite_contact_reference == employee_reference)
        .and_then(|s| s.zep_primary_calendar.as_deref())
        .filter(|u| !u.is_empty())
        .map(|u| u.to_string())
        .ok_or_else(|| "Kein Kalender für diesen Mitarbeiter konfiguriert.".to_string())
}

pub(super) fn load_caldav_session(
    app: &tauri::AppHandle,
    store: &LocalStore,
) -> Result<CaldavSession, AppError> {
    let credentials = load_zep_credentials()?;
    build_caldav_session(app, store, credentials)
}

pub(super) fn build_caldav_session(
    app: &tauri::AppHandle,
    store: &LocalStore,
    credentials: ZepStoredCredentials,
) -> Result<CaldavSession, AppError> {
    Ok(CaldavSession {
        client: crate::http::shared_client()?,
        username: credentials.username,
        password: credentials.password,
        base_url: store.api_endpoints.zep_caldav_root_url.clone(),
        telemetry: app
            .try_state::<TelemetryState>()
            .map(|state| std::sync::Arc::clone(state.recorder())),
        absence_urls: store
            .employee_settings
            .iter()
            .filter_map(|s| s.zep_absence_calendar.clone())
            .filter(|u| !u.is_empty())
            .collect(),
    })
}
