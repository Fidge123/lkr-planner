use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri_plugin_http::reqwest;

use super::caldav::{
    create_assignment_core, delete_assignment_core, fetch_calendar_events, update_assignment_core,
};
use super::events::{
    classify_event, map_absence_raw_events_for_week, resolve_event, sort_events_absences_first,
};
use super::types::{CalendarCellEvent, EmployeeWeekEvents, PendingEvent};

/// Loads all calendar events for every configured employee for the given week.
/// Returns one entry per employee that has a primary calendar configured.
/// Per-employee CalDAV failures are returned inline in `error`; only total failures
/// (store unavailable, bad date) return an `Err`.
#[tauri::command]
#[specta::specta]
pub async fn load_week_events(
    app: tauri::AppHandle,
    week_start: String,
) -> Result<Vec<EmployeeWeekEvents>, String> {
    let store = crate::integrations::local_store::load_local_store(app.clone())
        .map_err(|e| e.user_message)?;

    let week_start_date = NaiveDate::parse_from_str(&week_start, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Wochenstartdatum: {week_start}"))?;

    let (username, password) = match crate::integrations::zep::load_zep_credentials_from_keychain()
    {
        Ok(c) => (c.username, c.password),
        Err(e) => {
            // No credentials: return error for every employee with a calendar URL
            let results: Vec<EmployeeWeekEvents> = store
                .employee_settings
                .iter()
                .filter(|s| {
                    s.zep_primary_calendar
                        .as_deref()
                        .map(|u| !u.is_empty())
                        .unwrap_or(false)
                })
                .map(|s| EmployeeWeekEvents {
                    employee_reference: s.daylite_contact_reference.clone(),
                    events: vec![],
                    error: Some(e.user_message.clone()),
                })
                .collect();
            if results.is_empty() {
                eprintln!(
                    "load_week_events: ZEP credentials unavailable and no primary calendars configured: {}",
                    e.technical_message
                );
            }
            return Ok(results);
        }
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    // First pass: fetch CalDAV events for all employees concurrently (primary + absence per
    // employee are also concurrent via tokio::join!).
    let employee_futures: Vec<_> = store
        .employee_settings
        .iter()
        .filter_map(|setting| {
            let calendar_url = setting
                .zep_primary_calendar
                .as_deref()
                .filter(|u| !u.is_empty())
                .map(str::to_string)?;

            let absence_url = setting
                .zep_absence_calendar
                .as_deref()
                .filter(|u| !u.is_empty())
                .map(str::to_string);

            let employee_ref = setting.daylite_contact_reference.clone();
            let client = client.clone();
            let username = username.clone();
            let password = password.clone();

            Some(async move {
                let (primary_result, absence_result) = tokio::join!(
                    fetch_calendar_events(
                        &client,
                        &calendar_url,
                        &username,
                        &password,
                        week_start_date
                    ),
                    async {
                        match absence_url {
                            Some(ref url) => fetch_calendar_events(
                                &client,
                                url,
                                &username,
                                &password,
                                week_start_date,
                            )
                            .await
                            .ok(),
                            None => None,
                        }
                    }
                );
                (employee_ref, primary_result, absence_result)
            })
        })
        .collect();

    let fetch_results = futures::future::join_all(employee_futures).await;

    let mut pending_per_employee: Vec<(String, Vec<PendingEvent>, Vec<CalendarCellEvent>)> =
        Vec::new();
    let mut error_results: Vec<EmployeeWeekEvents> = Vec::new();

    for (employee_ref, primary_result, absence_result) in fetch_results {
        match primary_result {
            Ok(raw_events) => {
                let pending = raw_events.into_iter().map(classify_event).collect();
                let absence_events = absence_result
                    .map(|raw| map_absence_raw_events_for_week(raw, week_start_date))
                    .unwrap_or_default();
                pending_per_employee.push((employee_ref, pending, absence_events));
            }
            Err(error_msg) => {
                error_results.push(EmployeeWeekEvents {
                    employee_reference: employee_ref,
                    events: vec![],
                    error: Some(error_msg),
                });
            }
        }
    }

    // Collect unique project refs that are not in the local Daylite cache.
    let mut missing_refs: HashSet<String> = HashSet::new();
    for (_, pending_events, _) in &pending_per_employee {
        for event in pending_events {
            if let Some(ref project_ref) = event.project_ref {
                let in_cache = store
                    .daylite_cache
                    .projects
                    .iter()
                    .any(|p| p.reference == *project_ref);
                if !in_cache {
                    missing_refs.insert(project_ref.clone());
                }
            }
        }
    }

    // Second pass: fetch missing projects from the Daylite API (sequential, typically few).
    let mut api_results: HashMap<String, Option<(String, String)>> = HashMap::new();
    for project_ref in missing_refs {
        let result = crate::integrations::daylite::projects::fetch_project_by_reference(
            app.clone(),
            &project_ref,
        )
        .await;
        api_results.insert(project_ref, result);
    }

    // Build final results combining cache and API lookups.
    let mut results = error_results;
    for (employee_ref, pending_events, absence_events) in pending_per_employee {
        let mut events: Vec<CalendarCellEvent> = pending_events
            .into_iter()
            .map(|p| resolve_event(p, &store.daylite_cache, &api_results))
            .collect();
        events.extend(absence_events);
        // Deduplicate by UID to guard against CalDAV servers redelivering the same event.
        let mut seen_uids = HashSet::new();
        events.retain(|e| seen_uids.insert(e.uid.clone()));
        // Absence events are shown first within each day.
        sort_events_absences_first(&mut events);
        results.push(EmployeeWeekEvents {
            employee_reference: employee_ref,
            events,
            error: None,
        });
    }

    Ok(results)
}

/// Creates a new assignment event on the employee's primary CalDAV calendar.
/// Returns the CalDAV resource href (e.g. `{calendar_url}/{uid}.ics`) of the new event.
#[tauri::command]
#[specta::specta]
pub async fn create_assignment(
    app: tauri::AppHandle,
    employee_reference: String,
    date: String,
    project_ref: String,
    project_name: String,
) -> Result<String, String> {
    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let calendar_url = store
        .employee_settings
        .iter()
        .find(|s| s.daylite_contact_reference == employee_reference)
        .and_then(|s| s.zep_primary_calendar.as_deref())
        .filter(|u| !u.is_empty())
        .ok_or_else(|| "Kein Kalender für diesen Mitarbeiter konfiguriert.".to_string())?
        .to_string();

    let absence_urls = absence_calendar_urls(&store);

    let (username, password) = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map(|c| (c.username, c.password))
        .map_err(|e| e.user_message)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    create_assignment_core(
        &client,
        &calendar_url,
        &absence_urls,
        &username,
        &password,
        &date,
        &project_ref,
        &project_name,
    )
    .await
}

/// Collects every configured ZEP absence calendar URL across all employees.
/// Used to guard assignment writes against landing in an absence calendar.
fn absence_calendar_urls(store: &crate::integrations::local_store::LocalStore) -> Vec<String> {
    store
        .employee_settings
        .iter()
        .filter_map(|s| s.zep_absence_calendar.clone())
        .filter(|u| !u.is_empty())
        .collect()
}

/// Updates an existing assignment event in place using the stored CalDAV href.
#[tauri::command]
#[specta::specta]
pub async fn update_assignment(
    app: tauri::AppHandle,
    href: String,
    uid: String,
    date: String,
    project_ref: String,
    project_name: String,
) -> Result<(), String> {
    let (username, password) = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map(|c| (c.username, c.password))
        .map_err(|e| e.user_message)?;

    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let base_url = store.api_endpoints.zep_caldav_root_url.clone();
    let absence_urls = absence_calendar_urls(&store);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    update_assignment_core(
        &client,
        &href,
        &base_url,
        &absence_urls,
        &uid,
        &username,
        &password,
        &date,
        &project_ref,
        &project_name,
    )
    .await
}

/// Deletes an assignment event using the stored CalDAV href.
#[tauri::command]
#[specta::specta]
pub async fn delete_assignment(app: tauri::AppHandle, href: String) -> Result<(), String> {
    let (username, password) = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map(|c| (c.username, c.password))
        .map_err(|e| e.user_message)?;

    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let base_url = store.api_endpoints.zep_caldav_root_url.clone();
    let absence_urls = absence_calendar_urls(&store);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    delete_assignment_core(
        &client,
        &href,
        &base_url,
        &absence_urls,
        &username,
        &password,
    )
    .await
}
