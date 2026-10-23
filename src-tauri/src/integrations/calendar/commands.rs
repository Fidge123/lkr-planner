use chrono::NaiveDate;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeSet, HashMap, HashSet};

use super::appointment_sync::{
    event_worker, retry_failed_in_app, schedule_appointment_sync, WorkerDay,
};
use super::caldav::{
    create_assignment_core, delete_assignment_core, fetch_calendar_events, move_assignment_core,
    reorder_assignment_core, update_assignment_core, AssignmentWrite, CaldavSession,
};
use super::events::{
    classify_event, map_absence_raw_events_for_week, resolve_event, sort_week_events,
};
use super::protection::{refuse_protected_day_change, refuse_protected_event};
use super::types::{CalendarCellEvent, EmployeeWeekEvents, MoveAssignmentResult, PendingEvent};
use crate::error::AppError;
use crate::integrations::daylite::projects::ResolvedProject;
use crate::integrations::local_store::{EmployeeSetting, LocalStore};
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::{observe, record_failure};
use tauri::Manager;

/// Background loads are recorded apart so they do not count as time a user waited.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WeekLoadPurpose {
    Display,
    Prefetch,
    Refresh,
}

impl WeekLoadPurpose {
    fn operation(self) -> Operation {
        match self {
            Self::Display => Operation::LoadWeekEvents,
            Self::Prefetch => Operation::PrefetchWeekEvents,
            Self::Refresh => Operation::RefreshWeekEvents,
        }
    }
}

/// Without employee references, every employee with a calendar is loaded.
#[tauri::command]
#[specta::specta]
pub async fn load_week_events(
    app: tauri::AppHandle,
    week_start: String,
    employee_references: Option<Vec<String>>,
    purpose: WeekLoadPurpose,
) -> Result<Vec<EmployeeWeekEvents>, AppError> {
    observe(&app, purpose.operation(), Integration::Zep, async {
        let store = crate::integrations::local_store::load_local_store(app.clone())?;
        let only = employee_references.as_deref();

        let week_start_date = NaiveDate::parse_from_str(&week_start, "%Y-%m-%d")
            .map_err(|_| format!("Ungültiges Wochenstartdatum: {week_start}"))?;

        let credentials = match crate::integrations::zep::load_zep_credentials() {
            Ok(c) => c,
            Err(e) => {
                record_failure(&app, Operation::KeychainRead, Integration::Keychain, &e);
                return Ok(employees_with_error(&store, only, &e.user_message));
            }
        };

        let session = build_caldav_session(&app, &store, credentials)?;

        let (fetches, error_results) =
            fetch_week_for_employees(&store, only, &session, week_start_date).await;

        let cached_projects =
            crate::integrations::daylite::projects::cached_projects(&project_references(&fetches));
        Ok(assemble_week_events(
            fetches,
            error_results,
            &cached_projects,
        ))
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn create_assignment(
    app: tauri::AppHandle,
    input: CreateAssignmentInput,
) -> Result<String, AppError> {
    observe(&app, Operation::CreateAssignment, Integration::Zep, async {
        let store = crate::integrations::local_store::load_local_store(app.clone())?;
        let calendar_url = resolve_employee_calendar_url(&store, &input.employee_reference)?;
        let session = load_caldav_session(&app, &store)?;

        let write = AssignmentWrite {
            date: input.date,
            project_ref: input.project_ref,
            project_name: input.project_name,
            order_index: None,
        };
        let href = create_assignment_core(&session, &calendar_url, &write).await?;
        let worker = Some(input.employee_reference.as_str());
        schedule_appointment_sync(&app, worker_days(worker, [Some(write.date.as_str())]));
        Ok(href)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn update_assignment(
    app: tauri::AppHandle,
    input: UpdateAssignmentInput,
) -> Result<(), AppError> {
    observe(&app, Operation::UpdateAssignment, Integration::Zep, async {
        let store = crate::integrations::local_store::load_local_store(app.clone())?;
        let session = load_caldav_session(&app, &store)?;
        refuse_protected_event(
            &session,
            &input.href,
            input.override_protection,
            |reference| lookup_project(app.clone(), reference),
        )
        .await?;

        let write = AssignmentWrite {
            date: input.date,
            project_ref: input.project_ref,
            project_name: input.project_name,
            order_index: input.order_index,
        };
        let previous_date =
            update_assignment_core(&session, &input.href, &input.uid, &write).await?;
        let worker = event_worker(&app, &store, &session, &input.href);
        let dates = [Some(write.date.as_str()), previous_date.as_deref()];
        schedule_appointment_sync(&app, worker_days(worker.as_deref(), dates));
        Ok(())
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn move_assignment(
    app: tauri::AppHandle,
    input: MoveAssignmentInput,
) -> Result<MoveAssignmentResult, AppError> {
    observe(&app, Operation::MoveAssignment, Integration::Zep, async {
        let store = crate::integrations::local_store::load_local_store(app.clone())?;
        let target_calendar_url =
            resolve_employee_calendar_url(&store, &input.target_employee_reference)?;
        let session = load_caldav_session(&app, &store)?;
        refuse_protected_day_change(&session, &input.href, &input.date, |reference| {
            lookup_project(app.clone(), reference)
        })
        .await?;

        let write = AssignmentWrite {
            date: input.date,
            project_ref: input.project_ref,
            project_name: input.project_name,
            order_index: input.order_index,
        };
        let (moved, source_date) =
            move_assignment_core(&session, &input.href, &target_calendar_url, &write).await?;
        let source = event_worker(&app, &store, &session, &input.href);
        let touched = move_worker_days(
            &input.target_employee_reference,
            &write.date,
            source.as_deref(),
            source_date.as_deref(),
        );
        schedule_appointment_sync(&app, touched);
        Ok(moved)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn reorder_assignment(
    app: tauri::AppHandle,
    input: ReorderAssignmentInput,
) -> Result<(), AppError> {
    observe(
        &app,
        Operation::ReorderAssignment,
        Integration::Zep,
        async {
            let store = crate::integrations::local_store::load_local_store(app.clone())?;
            let session = load_caldav_session(&app, &store)?;
            Ok(reorder_assignment_core(
                &session,
                &input.href,
                &input.uid,
                &input.date,
                input.order_index,
            )
            .await?)
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_assignment(
    app: tauri::AppHandle,
    href: String,
    override_protection: bool,
) -> Result<(), AppError> {
    observe(&app, Operation::DeleteAssignment, Integration::Zep, async {
        let store = crate::integrations::local_store::load_local_store(app.clone())?;
        let session = load_caldav_session(&app, &store)?;
        refuse_protected_event(&session, &href, override_protection, |reference| {
            lookup_project(app.clone(), reference)
        })
        .await?;

        let previous_date = delete_assignment_core(&session, &href).await?;
        let worker = event_worker(&app, &store, &session, &href);
        schedule_appointment_sync(
            &app,
            worker_days(worker.as_deref(), [previous_date.as_deref()]),
        );
        Ok(())
    })
    .await
}

/// Returns nothing; the failures event reports the worker days still failing.
#[tauri::command]
#[specta::specta]
pub async fn retry_daylite_appointment_sync(app: tauri::AppHandle) {
    retry_failed_in_app(&app).await;
}

// One unit is two requests, primary and absence, so the roster fans out to twice this.
const EMPLOYEE_FETCH_CONCURRENCY: usize = 4;

struct EmployeeFetch {
    employee_reference: String,
    pending: Vec<PendingEvent>,
    absences: Vec<CalendarCellEvent>,
}

fn employees_to_load<'a>(
    store: &'a LocalStore,
    only: Option<&'a [String]>,
) -> impl Iterator<Item = &'a EmployeeSetting> {
    store.employee_settings.iter().filter(move |s| {
        s.zep_primary_calendar
            .as_deref()
            .is_some_and(|u| !u.is_empty())
            && only.is_none_or(|refs| refs.contains(&s.daylite_contact_reference))
    })
}

fn employees_with_error(
    store: &LocalStore,
    only: Option<&[String]>,
    message: &str,
) -> Vec<EmployeeWeekEvents> {
    employees_to_load(store, only)
        .map(|s| EmployeeWeekEvents {
            employee_reference: s.daylite_contact_reference.clone(),
            events: vec![],
            error: Some(message.to_string()),
        })
        .collect()
}

async fn fetch_week_for_employees(
    store: &LocalStore,
    only: Option<&[String]>,
    session: &CaldavSession,
    week_start: NaiveDate,
) -> (Vec<EmployeeFetch>, Vec<EmployeeWeekEvents>) {
    let employee_futures: Vec<_> = employees_to_load(store, only)
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

    let fetch_results: Vec<_> = futures::stream::iter(employee_futures)
        .buffer_unordered(EMPLOYEE_FETCH_CONCURRENCY)
        .collect()
        .await;

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

fn project_references(fetches: &[EmployeeFetch]) -> HashSet<String> {
    let mut references: HashSet<String> = HashSet::new();
    for fetch in fetches {
        for event in &fetch.pending {
            if let Some(ref project_ref) = event.project_ref {
                references.insert(project_ref.clone());
            }
        }
    }

    references
}

fn assemble_week_events(
    fetches: Vec<EmployeeFetch>,
    error_results: Vec<EmployeeWeekEvents>,
    cached_projects: &HashMap<String, ResolvedProject>,
) -> Vec<EmployeeWeekEvents> {
    let mut results = error_results;
    for fetch in fetches {
        let mut events: Vec<CalendarCellEvent> = fetch
            .pending
            .into_iter()
            .map(|p| resolve_event(p, cached_projects))
            .collect();
        events.extend(fetch.absences);
        // Deduplicate by UID to guard against CalDAV servers redelivering the same event.
        let mut seen_uids = HashSet::new();
        events.retain(|e| seen_uids.insert(e.uid.clone()));
        sort_week_events(&mut events);
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
    /// The user deliberately unlocked a fixed appointment for this one write.
    pub override_protection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MoveAssignmentInput {
    pub href: String,
    pub target_employee_reference: String,
    pub date: String,
    pub project_ref: String,
    pub project_name: String,
    pub order_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReorderAssignmentInput {
    pub href: String,
    pub uid: String,
    pub date: String,
    pub order_index: u32,
}

async fn lookup_project(app: tauri::AppHandle, reference: String) -> Option<ResolvedProject> {
    crate::integrations::daylite::projects::fetch_project_by_reference(app, &reference).await
}

fn worker_days<'a>(
    worker: Option<&str>,
    dates: impl IntoIterator<Item = Option<&'a str>>,
) -> Vec<WorkerDay> {
    let Some(worker) = worker else {
        return Vec::new();
    };
    dates
        .into_iter()
        .flatten()
        .filter_map(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|date| WorkerDay {
            worker: worker.to_string(),
            date,
        })
        .collect()
}

/// Target first, so the source's unlink cannot delete the appointment the target is joining.
fn move_worker_days(
    target: &str,
    target_date: &str,
    source: Option<&str>,
    source_date: Option<&str>,
) -> Vec<WorkerDay> {
    let mut touched = worker_days(Some(target), [Some(target_date)]);
    touched.extend(worker_days(source, [source_date]));
    touched
}

pub(super) fn resolve_employee_calendar_url(
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

pub(super) fn load_caldav_session(
    app: &tauri::AppHandle,
    store: &crate::integrations::local_store::LocalStore,
) -> Result<CaldavSession, AppError> {
    let credentials = crate::integrations::zep::load_zep_credentials()?;
    build_caldav_session(app, store, credentials)
}

fn build_caldav_session(
    app: &tauri::AppHandle,
    store: &crate::integrations::local_store::LocalStore,
    credentials: crate::integrations::zep::ZepStoredCredentials,
) -> Result<CaldavSession, AppError> {
    Ok(CaldavSession {
        client: crate::http::shared_client()?,
        username: credentials.username,
        password: credentials.password,
        base_url: store.api_endpoints.zep_caldav_root_url.clone(),
        telemetry: app
            .try_state::<crate::integrations::telemetry::state::TelemetryState>()
            .map(|state| std::sync::Arc::clone(state.recorder())),
        absence_urls: store
            .employee_settings
            .iter()
            .filter_map(|s| s.zep_absence_calendar.clone())
            .filter(|u| !u.is_empty())
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(employees: &[(&str, Option<&str>)]) -> LocalStore {
        LocalStore {
            employee_settings: employees
                .iter()
                .map(|(reference, calendar)| EmployeeSetting {
                    daylite_contact_reference: reference.to_string(),
                    zep_primary_calendar: calendar.map(str::to_string),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn references(selected: Vec<&EmployeeSetting>) -> Vec<&str> {
        selected
            .iter()
            .map(|s| s.daylite_contact_reference.as_str())
            .collect()
    }

    #[test]
    fn without_a_filter_every_employee_with_a_primary_calendar_is_loaded() {
        let store = store_with(&[
            ("/v1/contacts/1", Some("https://zep/cal-1")),
            ("/v1/contacts/2", None),
            ("/v1/contacts/3", Some("")),
            ("/v1/contacts/4", Some("https://zep/cal-4")),
        ]);

        let selected = employees_to_load(&store, None).collect();

        assert_eq!(references(selected), ["/v1/contacts/1", "/v1/contacts/4"]);
    }

    #[test]
    fn a_filter_limits_the_load_to_the_listed_employees() {
        let store = store_with(&[
            ("/v1/contacts/1", Some("https://zep/cal-1")),
            ("/v1/contacts/2", Some("https://zep/cal-2")),
            ("/v1/contacts/3", None),
        ]);
        let only = ["/v1/contacts/2".to_string(), "/v1/contacts/3".to_string()];

        let selected = employees_to_load(&store, Some(&only)).collect();

        assert_eq!(references(selected), ["/v1/contacts/2"]);
    }

    #[test]
    fn each_load_purpose_records_its_own_operation() {
        assert_eq!(
            WeekLoadPurpose::Display.operation(),
            Operation::LoadWeekEvents
        );
        assert_eq!(
            WeekLoadPurpose::Prefetch.operation(),
            Operation::PrefetchWeekEvents
        );
        assert_eq!(
            WeekLoadPurpose::Refresh.operation(),
            Operation::RefreshWeekEvents
        );
    }

    fn on(worker: &str, date: &str) -> WorkerDay {
        WorkerDay {
            worker: worker.to_string(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        }
    }

    #[test]
    fn a_write_touches_the_workers_day_it_left_and_the_day_it_entered() {
        let cases = [
            (
                "two days",
                worker_days(
                    Some("/v1/contacts/1"),
                    [Some("2026-11-03"), Some("2026-11-02")],
                ),
                vec![
                    on("/v1/contacts/1", "2026-11-02"),
                    on("/v1/contacts/1", "2026-11-03"),
                ],
            ),
            (
                "one day once",
                worker_days(
                    Some("/v1/contacts/1"),
                    [Some("2026-11-02"), Some("2026-11-02")],
                ),
                vec![on("/v1/contacts/1", "2026-11-02")],
            ),
            (
                "an unknown or unreadable date",
                worker_days(Some("/v1/contacts/1"), [None, Some("kein-datum")]),
                vec![],
            ),
            (
                "an unknown worker",
                worker_days(None, [Some("2026-11-02")]),
                vec![],
            ),
        ];

        for (label, touched, expected) in cases {
            assert_eq!(touched, expected, "case: {label}");
        }
    }

    #[test]
    fn a_move_synchronizes_the_target_worker_before_the_source_worker() {
        assert_eq!(
            move_worker_days(
                "/v1/contacts/2",
                "2026-11-02",
                Some("/v1/contacts/1"),
                Some("2026-11-02"),
            ),
            [
                on("/v1/contacts/2", "2026-11-02"),
                on("/v1/contacts/1", "2026-11-02"),
            ]
        );
    }

    #[test]
    fn update_input_carries_the_override_the_frontend_sends() {
        let input: UpdateAssignmentInput = serde_json::from_str(
            r#"{"href":"/cal/uid-1.ics","uid":"uid-1","date":"2026-05-06",
                "projectRef":"/v1/projects/1","projectName":"Projekt Nord",
                "orderIndex":null,"overrideProtection":true}"#,
        )
        .expect("update input parses");

        assert!(input.override_protection);
    }
}
