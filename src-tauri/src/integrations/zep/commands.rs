use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;

#[tauri::command]
#[specta::specta]
pub async fn zep_test_credentials(
    app: tauri::AppHandle,
    root_url: String,
    username: String,
    password: String,
) -> Result<ZepCredentialTestResult, AppError> {
    observe(
        &app,
        Operation::ZepTestCredentials,
        Integration::Zep,
        async {
            let root_url = normalized_root_url(&root_url)?;

            let body = propfind(&root_url, &username, &password).await?;
            let calendars = parse_propfind_calendars(&body, &root_url);

            Ok(ZepCredentialTestResult {
                calendar_count: calendars.len() as u32,
            })
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn zep_save_credentials(
    app: tauri::AppHandle,
    root_url: String,
    username: String,
    password: String,
) -> Result<(), AppError> {
    observe(
        &app,
        Operation::ZepSaveCredentials,
        Integration::Zep,
        async {
            let root_url = normalized_root_url(&root_url)?;

            // Write the store first so a keychain failure leaves no orphaned credential entry.
            crate::integrations::local_store::update_local_store(&app, |store| {
                store.api_endpoints.zep_caldav_root_url = root_url
            })?;

            save_zep_credentials_to_keychain(&username, &password)?;

            Ok(())
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn zep_load_credentials(
    app: tauri::AppHandle,
) -> Result<Option<ZepCredentialsInfo>, AppError> {
    observe(
        &app,
        Operation::ZepLoadCredentials,
        Integration::Zep,
        async {
            let store = crate::integrations::local_store::load_local_store(app.clone())?;

            let root_url = store.api_endpoints.zep_caldav_root_url.trim().to_string();
            if root_url.is_empty() {
                return Ok(None);
            }

            match load_zep_credentials() {
                Ok(creds) => Ok(Some(ZepCredentialsInfo {
                    root_url,
                    username: creds.username,
                })),
                Err(e) if e.code == AppErrorCode::MissingCredentials => Ok(None),
                Err(e) => Err(e),
            }
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn zep_discover_calendars(app: tauri::AppHandle) -> Result<Vec<ZepCalendar>, AppError> {
    observe(
        &app,
        Operation::ZepDiscoverCalendars,
        Integration::Zep,
        async {
            let store = crate::integrations::local_store::load_local_store(app.clone())?;

            let root_url = store.api_endpoints.zep_caldav_root_url.trim().to_string();
            if root_url.is_empty() {
                return Err(AppError::new(
                    AppErrorCode::MissingCredentials,
                    "ZEP CalDAV-URL nicht konfiguriert. Bitte ZEP-Verbindung einrichten.",
                    "zep_caldav_root_url is empty in local store",
                ));
            }

            let creds = load_zep_credentials()?;
            let body = propfind(&root_url, &creds.username, &creds.password).await?;
            let calendars = parse_propfind_calendars(&body, &root_url);

            Ok(calendars)
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn zep_save_and_test_calendar(
    app: tauri::AppHandle,
    daylite_contact_reference: String,
    source: IcalSource,
    calendar_url: Option<String>,
) -> Result<ZepCalendarTestResult, AppError> {
    observe(
        &app,
        Operation::ZepSaveAndTestCalendar,
        Integration::Zep,
        async {
            let calendar_url = calendar_url
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string);

            let mut synced = crate::integrations::local_store::load_local_store(app.clone())?;
            sync_calendar_to_daylite(
                &mut synced,
                &daylite_contact_reference,
                &source,
                &calendar_url,
            )
            .await?;
            crate::integrations::local_store::update_local_store(&app, |store| {
                store.daylite_cache = synced.daylite_cache;
                store_calendar_url(store, &daylite_contact_reference, &source, &calendar_url);
            })?;

            // Clearing a calendar reports success without a connection test.
            let Some(ref cal_url) = calendar_url else {
                return Ok(ZepCalendarTestResult {
                    success: true,
                    timestamp: current_timestamp(),
                    error_message: None,
                });
            };

            let result = run_calendar_test(cal_url).await?;
            crate::integrations::local_store::update_local_store(&app, |store| {
                store_test_result(store, &daylite_contact_reference, &source, &result)
            })?;

            Ok(result)
        },
    )
    .await
}

use super::caldav::{parse_propfind_calendars, probe_calendar, propfind};
use super::credentials::{load_zep_credentials, save_zep_credentials_to_keychain};
use super::settings::{current_timestamp, find_or_default_setting, update_setting};
use super::types::{
    IcalSource, ZepCalendar, ZepCalendarTestResult, ZepCredentialTestResult, ZepCredentialsInfo,
};
use crate::error::{AppError, AppErrorCode};
use crate::integrations::daylite::contacts::{
    sync_contact_ical_urls, DayliteUpdateContactIcalUrlsInput,
};

fn normalized_root_url(root_url: &str) -> Result<String, AppError> {
    let root_url = root_url.trim().trim_end_matches('/');
    if root_url.is_empty() {
        return Err(AppError::new(
            AppErrorCode::InvalidConfiguration,
            "Die ZEP CalDAV-URL darf nicht leer sein.",
            "root_url is empty",
        ));
    }
    Ok(root_url.to_string())
}

/// An empty URL string removes the entry in Daylite via `normalize_non_empty`.
async fn sync_calendar_to_daylite(
    store: &mut crate::integrations::local_store::LocalStore,
    contact_reference: &str,
    source: &IcalSource,
    calendar_url: &Option<String>,
) -> Result<(), AppError> {
    let current_setting = find_or_default_setting(&store.employee_settings, contact_reference);
    let (primary_url, absence_url) = match source {
        IcalSource::Primary => (
            calendar_url.clone().unwrap_or_default(),
            current_setting
                .zep_absence_calendar
                .clone()
                .unwrap_or_default(),
        ),
        IcalSource::Absence => (
            current_setting
                .zep_primary_calendar
                .clone()
                .unwrap_or_default(),
            calendar_url.clone().unwrap_or_default(),
        ),
    };

    sync_contact_ical_urls(
        store,
        DayliteUpdateContactIcalUrlsInput {
            contact_reference: contact_reference.to_string(),
            primary_ical_url: primary_url,
            absence_ical_url: absence_url,
        },
    )
    .await
    .map_err(|e| {
        AppError::new(
            AppErrorCode::DayliteSyncFailed,
            format!("Daylite-Synchronisation fehlgeschlagen: {}", e.user_message),
            e.technical_message,
        )
    })
}

/// A changed URL invalidates the previous test result.
fn store_calendar_url(
    store: &mut crate::integrations::local_store::LocalStore,
    contact_reference: &str,
    source: &IcalSource,
    calendar_url: &Option<String>,
) {
    update_setting(
        &mut store.employee_settings,
        contact_reference,
        |s| match source {
            IcalSource::Primary => {
                s.zep_primary_calendar = calendar_url.clone();
                s.primary_ical_last_tested_at = None;
                s.primary_ical_last_test_passed = None;
            }
            IcalSource::Absence => {
                s.zep_absence_calendar = calendar_url.clone();
                s.absence_ical_last_tested_at = None;
                s.absence_ical_last_test_passed = None;
            }
        },
    );
}

async fn run_calendar_test(calendar_url: &str) -> Result<ZepCalendarTestResult, AppError> {
    let creds = load_zep_credentials()?;
    let timestamp = current_timestamp();
    let (success, error_message) =
        match probe_calendar(calendar_url, &creds.username, &creds.password).await {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.user_message.clone())),
        };

    Ok(ZepCalendarTestResult {
        success,
        timestamp,
        error_message,
    })
}

fn store_test_result(
    store: &mut crate::integrations::local_store::LocalStore,
    contact_reference: &str,
    source: &IcalSource,
    result: &ZepCalendarTestResult,
) {
    update_setting(
        &mut store.employee_settings,
        contact_reference,
        |s| match source {
            IcalSource::Primary => {
                s.primary_ical_last_tested_at = Some(result.timestamp.clone());
                s.primary_ical_last_test_passed = Some(result.success);
            }
            IcalSource::Absence => {
                s.absence_ical_last_tested_at = Some(result.timestamp.clone());
                s.absence_ical_last_test_passed = Some(result.success);
            }
        },
    );
}
