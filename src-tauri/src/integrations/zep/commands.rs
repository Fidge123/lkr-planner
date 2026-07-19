use super::caldav::{parse_propfind_calendars, probe_calendar, propfind};
use super::credentials::{load_zep_credentials_from_keychain, save_zep_credentials_to_keychain};
use super::settings::{current_timestamp, find_or_default_setting, update_setting};
use super::types::{
    IcalSource, ZepCalendar, ZepCalendarTestResult, ZepCredentialTestResult, ZepCredentialsInfo,
    ZepError, ZepErrorCode,
};
use crate::integrations::daylite::contacts::{
    sync_contact_ical_urls, DayliteUpdateContactIcalUrlsInput,
};

#[tauri::command]
#[specta::specta]
pub fn zep_save_credentials(
    app: tauri::AppHandle,
    root_url: String,
    username: String,
    password: String,
) -> Result<(), ZepError> {
    let root_url = root_url.trim().trim_end_matches('/').to_string();
    if root_url.is_empty() {
        return Err(ZepError::new(
            ZepErrorCode::InvalidConfiguration,
            "Die ZEP CalDAV-URL darf nicht leer sein.",
            "root_url is empty",
        ));
    }

    // Write the store first so that a keychain failure leaves no orphaned credential entry.
    let mut store = crate::integrations::local_store::load_local_store(app.clone())?;
    store.api_endpoints.zep_caldav_root_url = root_url;
    crate::integrations::local_store::save_local_store(app, store)?;

    save_zep_credentials_to_keychain(&username, &password)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn zep_load_credentials(app: tauri::AppHandle) -> Result<Option<ZepCredentialsInfo>, ZepError> {
    let store = crate::integrations::local_store::load_local_store(app)?;

    let root_url = store.api_endpoints.zep_caldav_root_url.trim().to_string();
    if root_url.is_empty() {
        return Ok(None);
    }

    match load_zep_credentials_from_keychain() {
        Ok(creds) => Ok(Some(ZepCredentialsInfo {
            root_url,
            username: creds.username,
        })),
        Err(e) if e.code == ZepErrorCode::MissingCredentials => Ok(None),
        Err(e) => Err(e),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn zep_test_credentials(
    root_url: String,
    username: String,
    password: String,
) -> Result<ZepCredentialTestResult, ZepError> {
    let root_url = root_url.trim().trim_end_matches('/').to_string();
    if root_url.is_empty() {
        return Err(ZepError::new(
            ZepErrorCode::InvalidConfiguration,
            "Die ZEP CalDAV-URL darf nicht leer sein.",
            "root_url is empty",
        ));
    }

    let body = propfind(&root_url, &username, &password).await?;
    let calendars = parse_propfind_calendars(&body, &root_url);

    Ok(ZepCredentialTestResult {
        calendar_count: calendars.len() as u32,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn zep_discover_calendars(app: tauri::AppHandle) -> Result<Vec<ZepCalendar>, ZepError> {
    let store = crate::integrations::local_store::load_local_store(app)?;

    let root_url = store.api_endpoints.zep_caldav_root_url.trim().to_string();
    if root_url.is_empty() {
        return Err(ZepError::new(
            ZepErrorCode::MissingCredentials,
            "ZEP CalDAV-URL nicht konfiguriert. Bitte ZEP-Verbindung einrichten.",
            "zep_caldav_root_url is empty in local store",
        ));
    }

    let creds = load_zep_credentials_from_keychain()?;
    let body = propfind(&root_url, &creds.username, &creds.password).await?;
    let calendars = parse_propfind_calendars(&body, &root_url);

    Ok(calendars)
}

#[tauri::command]
#[specta::specta]
pub async fn zep_save_and_test_calendar(
    app: tauri::AppHandle,
    daylite_contact_reference: String,
    source: IcalSource,
    calendar_url: Option<String>,
) -> Result<ZepCalendarTestResult, ZepError> {
    let calendar_url = calendar_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);

    // Load local store once: all in-memory mutations use this copy.
    let mut store = crate::integrations::local_store::load_local_store(app.clone())?;

    sync_calendar_to_daylite(
        &mut store,
        &daylite_contact_reference,
        &source,
        &calendar_url,
    )
    .await?;
    store_calendar_url(
        &mut store,
        &daylite_contact_reference,
        &source,
        &calendar_url,
    );
    crate::integrations::local_store::save_local_store(app.clone(), store.clone())?;

    // Clearing a calendar reports success without a connection test.
    let Some(ref cal_url) = calendar_url else {
        return Ok(ZepCalendarTestResult {
            success: true,
            timestamp: current_timestamp(),
            error_message: None,
        });
    };

    let result = run_calendar_test(cal_url).await?;
    store_test_result(&mut store, &daylite_contact_reference, &source, &result);
    crate::integrations::local_store::save_local_store(app, store)?;

    Ok(result)
}

/// An empty URL string removes the entry in Daylite via `normalize_non_empty`.
async fn sync_calendar_to_daylite(
    store: &mut crate::integrations::local_store::LocalStore,
    contact_reference: &str,
    source: &IcalSource,
    calendar_url: &Option<String>,
) -> Result<(), ZepError> {
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
        ZepError::new(
            ZepErrorCode::DayliteSyncFailed,
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

async fn run_calendar_test(calendar_url: &str) -> Result<ZepCalendarTestResult, ZepError> {
    let creds = load_zep_credentials_from_keychain()?;
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
