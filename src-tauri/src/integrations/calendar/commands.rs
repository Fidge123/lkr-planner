use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri_plugin_http::reqwest;

use super::caldav::{
    create_assignment_core, delete_assignment_core, fetch_calendar_events, move_assignment_core,
    reorder_assignment_core, update_assignment_core, AssignmentWrite, CaldavSession,
};
use super::events::{
    classify_event, map_absence_raw_events_for_week, resolve_event, sort_events_absences_first,
};
use super::protection::{refuse_protected_day_change, refuse_protected_event};
use super::types::{CalendarCellEvent, EmployeeWeekEvents, MoveAssignmentResult, PendingEvent};
use crate::integrations::daylite::projects::ResolvedProject;
use crate::integrations::local_store::{DayliteCache, LocalStore};

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

    let credentials = match crate::integrations::zep::load_zep_credentials_from_keychain() {
        Ok(c) => c,
        Err(e) => {
            let results = employees_with_error(&store, &e.user_message);
            if results.is_empty() {
                eprintln!(
                    "load_week_events: ZEP credentials unavailable and no primary calendars configured: {}",
                    e.technical_message
                );
            }
            return Ok(results);
        }
    };

    let session = build_caldav_session(&store, credentials)?;

    let (fetches, error_results) =
        fetch_week_for_employees(&store, &session, week_start_date).await;

    let api_results = fetch_uncached_projects(app.clone(), &store, &fetches).await;
    let category_colors =
        crate::integrations::daylite::categories::fetch_project_category_colors(app).await;

    Ok(assemble_week_events(
        fetches,
        error_results,
        &store.daylite_cache,
        &api_results,
        &category_colors,
    ))
}

struct EmployeeFetch {
    employee_reference: String,
    pending: Vec<PendingEvent>,
    absences: Vec<CalendarCellEvent>,
}

fn employees_with_error(store: &LocalStore, message: &str) -> Vec<EmployeeWeekEvents> {
    store
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
            error: Some(message.to_string()),
        })
        .collect()
}

async fn fetch_week_for_employees(
    store: &LocalStore,
    session: &CaldavSession,
    week_start: NaiveDate,
) -> (Vec<EmployeeFetch>, Vec<EmployeeWeekEvents>) {
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

            Some(async move {
                let (primary_result, absence_result) = tokio::join!(
                    fetch_calendar_events(session, &calendar_url, week_start),
                    async {
                        match absence_url {
                            Some(ref url) => {
                                fetch_calendar_events(session, url, week_start).await.ok()
                            }
                            None => None,
                        }
                    }
                );
                (employee_ref, primary_result, absence_result)
            })
        })
        .collect();

    let fetch_results = futures::future::join_all(employee_futures).await;

    let mut fetches = Vec::new();
    let mut error_results = Vec::new();

    for (employee_reference, primary_result, absence_result) in fetch_results {
        match primary_result {
            Ok(raw_events) => {
                fetches.push(EmployeeFetch {
                    employee_reference,
                    pending: raw_events.iter().map(classify_event).collect(),
                    absences: absence_result
                        .map(|raw| map_absence_raw_events_for_week(raw, week_start))
                        .unwrap_or_default(),
                });
            }
            Err(error_msg) => {
                error_results.push(EmployeeWeekEvents {
                    employee_reference,
                    events: vec![],
                    error: Some(error_msg),
                });
            }
        }
    }

    (fetches, error_results)
}

async fn fetch_uncached_projects(
    app: tauri::AppHandle,
    store: &LocalStore,
    fetches: &[EmployeeFetch],
) -> HashMap<String, Option<ResolvedProject>> {
    let mut missing_refs: HashSet<String> = HashSet::new();
    for fetch in fetches {
        for event in &fetch.pending {
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

    let mut api_results = HashMap::new();
    for project_ref in missing_refs {
        let result = crate::integrations::daylite::projects::fetch_project_by_reference(
            app.clone(),
            &project_ref,
        )
        .await;
        api_results.insert(project_ref, result);
    }

    api_results
}

fn assemble_week_events(
    fetches: Vec<EmployeeFetch>,
    error_results: Vec<EmployeeWeekEvents>,
    cache: &DayliteCache,
    api_results: &HashMap<String, Option<ResolvedProject>>,
    category_colors: &HashMap<String, String>,
) -> Vec<EmployeeWeekEvents> {
    let mut results = error_results;
    for fetch in fetches {
        let mut events: Vec<CalendarCellEvent> = fetch
            .pending
            .into_iter()
            .map(|p| resolve_event(p, cache, api_results, category_colors))
            .collect();
        events.extend(fetch.absences);
        // Deduplicate by UID to guard against CalDAV servers redelivering the same event.
        let mut seen_uids = HashSet::new();
        events.retain(|e| seen_uids.insert(e.uid.clone()));
        sort_events_absences_first(&mut events);
        results.push(EmployeeWeekEvents {
            employee_reference: fetch.employee_reference,
            events,
            error: None,
        });
    }

    results
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssignmentInput {
    pub employee_reference: String,
    pub date: String,
    pub project_ref: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAssignmentInput {
    pub href: String,
    pub uid: String,
    pub date: String,
    pub project_ref: String,
    pub project_name: String,
    /// Position among the target day's assignments. None keeps the assignment where it is.
    pub order_index: Option<u32>,
    /// Set when the user deliberately unlocked a fixed appointment in the modal.
    pub override_protection: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn create_assignment(
    app: tauri::AppHandle,
    input: CreateAssignmentInput,
) -> Result<String, String> {
    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let calendar_url = resolve_employee_calendar_url(&store, &input.employee_reference)?;

    let session = load_caldav_session(&store)?;

    create_assignment_core(
        &session,
        &calendar_url,
        &AssignmentWrite {
            date: input.date,
            project_ref: input.project_ref,
            project_name: input.project_name,
            order_index: None,
        },
    )
    .await
}

fn resolve_employee_calendar_url(
    store: &crate::integrations::local_store::LocalStore,
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

fn load_caldav_session(
    store: &crate::integrations::local_store::LocalStore,
) -> Result<CaldavSession, String> {
    let credentials = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map_err(|e| e.user_message)?;
    build_caldav_session(store, credentials)
}

fn build_caldav_session(
    store: &crate::integrations::local_store::LocalStore,
    credentials: crate::integrations::zep::ZepStoredCredentials,
) -> Result<CaldavSession, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    Ok(CaldavSession {
        client,
        username: credentials.username,
        password: credentials.password,
        base_url: store.api_endpoints.zep_caldav_root_url.clone(),
        absence_urls: store
            .employee_settings
            .iter()
            .filter_map(|s| s.zep_absence_calendar.clone())
            .filter(|u| !u.is_empty())
            .collect(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn update_assignment(
    app: tauri::AppHandle,
    input: UpdateAssignmentInput,
) -> Result<(), String> {
    let store = crate::integrations::local_store::load_local_store(app.clone())
        .map_err(|e| e.user_message)?;
    let session = load_caldav_session(&store)?;

    if !input.override_protection {
        refuse_protected_event(app, &session, &input.href).await?;
    }

    update_assignment_core(
        &session,
        &input.href,
        &input.uid,
        &AssignmentWrite {
            date: input.date,
            project_ref: input.project_ref,
            project_name: input.project_name,
            order_index: input.order_index,
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn move_assignment(
    app: tauri::AppHandle,
    href: String,
    target_employee_reference: String,
    date: String,
    project_ref: String,
    project_name: String,
    order_index: Option<u32>,
) -> Result<MoveAssignmentResult, String> {
    let store = crate::integrations::local_store::load_local_store(app.clone())
        .map_err(|e| e.user_message)?;

    let target_calendar_url = resolve_employee_calendar_url(&store, &target_employee_reference)?;

    let session = load_caldav_session(&store)?;

    refuse_protected_day_change(app, &session, &href, &date).await?;

    move_assignment_core(
        &session,
        &href,
        &target_calendar_url,
        &AssignmentWrite {
            date,
            project_ref,
            project_name,
            order_index,
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn reorder_assignment(
    app: tauri::AppHandle,
    href: String,
    uid: String,
    date: String,
    order_index: u32,
) -> Result<(), String> {
    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;
    let session = load_caldav_session(&store)?;

    reorder_assignment_core(&session, &href, &uid, &date, order_index).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_assignment(
    app: tauri::AppHandle,
    href: String,
    override_protection: bool,
) -> Result<(), String> {
    let store = crate::integrations::local_store::load_local_store(app.clone())
        .map_err(|e| e.user_message)?;
    let session = load_caldav_session(&store)?;

    if !override_protection {
        refuse_protected_event(app, &session, &href).await?;
    }

    delete_assignment_core(&session, &href).await
}
