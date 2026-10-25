//! Employee calendars link Daylite contacts to ZEP calendars, so their commands use both integrations.

use crate::error::{AppError, AppErrorCode};
use crate::integrations::daylite::contacts::types::PlanningContactRecord;
use crate::integrations::daylite::contacts::{
    cache_contacts, fetch_planning_contacts, reconcile_employee_calendars_from_contacts,
    sync_contact_ical_urls, DayliteUpdateContactIcalUrlsInput,
};
use crate::integrations::local_store::{
    load_local_store, update_local_store, EmployeeSetting, LocalStore,
};
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;
use crate::integrations::zep::types::{IcalSource, ZepCalendarTestResult};
use crate::integrations::zep::{current_timestamp, run_calendar_test, test_untested_calendar_urls};

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, AppError> {
    observe(
        &app,
        Operation::DayliteListContacts,
        Integration::Daylite,
        async {
            let contacts = fetch_planning_contacts(&app).await?;

            let mut settings = load_local_store(app.clone())?.employee_settings;
            reconcile_employee_calendars_from_contacts(&mut settings, &contacts);
            test_untested_calendar_urls(&mut settings).await;
            update_local_store(&app, |store| {
                cache_contacts(store, &contacts);
                store.employee_settings = settings;
            })?;

            Ok(contacts)
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

            let mut synced = load_local_store(app.clone())?;
            sync_calendar_to_daylite(
                &mut synced,
                &daylite_contact_reference,
                &source,
                &calendar_url,
            )
            .await?;
            update_local_store(&app, |store| {
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
            update_local_store(&app, |store| {
                store_test_result(store, &daylite_contact_reference, &source, &result)
            })?;

            Ok(result)
        },
    )
    .await
}

/// An empty URL string removes the entry in Daylite via `normalize_non_empty`.
async fn sync_calendar_to_daylite(
    store: &mut LocalStore,
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
    store: &mut LocalStore,
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

fn store_test_result(
    store: &mut LocalStore,
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

fn find_or_default_setting(
    settings: &[EmployeeSetting],
    daylite_contact_reference: &str,
) -> EmployeeSetting {
    settings
        .iter()
        .find(|s| s.daylite_contact_reference == daylite_contact_reference)
        .cloned()
        .unwrap_or_else(|| EmployeeSetting {
            daylite_contact_reference: daylite_contact_reference.to_string(),
            ..Default::default()
        })
}

fn update_setting(
    settings: &mut Vec<EmployeeSetting>,
    daylite_contact_reference: &str,
    update: impl FnOnce(&mut EmployeeSetting),
) {
    if let Some(setting) = settings
        .iter_mut()
        .find(|s| s.daylite_contact_reference == daylite_contact_reference)
    {
        update(setting);
    } else {
        let mut new_setting = EmployeeSetting {
            daylite_contact_reference: daylite_contact_reference.to_string(),
            ..Default::default()
        };
        update(&mut new_setting);
        settings.push(new_setting);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_setting_creates_new_entry_when_not_found() {
        let mut settings: Vec<EmployeeSetting> = vec![];
        update_setting(&mut settings, "/v1/contacts/42", |s| {
            s.zep_primary_calendar = Some("https://app.zep.de/cal/".to_string());
        });
        assert_eq!(settings.len(), 1);
        assert_eq!(
            settings[0].zep_primary_calendar,
            Some("https://app.zep.de/cal/".to_string())
        );
    }

    #[test]
    fn update_setting_updates_existing_entry_and_clears_timestamp() {
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/42".to_string(),
            primary_ical_last_tested_at: Some("2026-01-01T00:00:00Z".to_string()),
            primary_ical_last_test_passed: Some(true),
            ..Default::default()
        }];
        update_setting(&mut settings, "/v1/contacts/42", |s| {
            s.zep_primary_calendar = Some("https://cal.example/".to_string());
            s.primary_ical_last_tested_at = None;
        });
        assert_eq!(settings.len(), 1);
        assert_eq!(
            settings[0].zep_primary_calendar,
            Some("https://cal.example/".to_string())
        );
        assert_eq!(settings[0].primary_ical_last_tested_at, None);
    }
}
