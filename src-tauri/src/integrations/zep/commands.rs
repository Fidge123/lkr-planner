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

use super::caldav::{parse_propfind_calendars, propfind};
use super::credentials::{load_zep_credentials, save_zep_credentials_to_keychain};
use super::types::{ZepCalendar, ZepCredentialTestResult, ZepCredentialsInfo};
use crate::error::{AppError, AppErrorCode};

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
