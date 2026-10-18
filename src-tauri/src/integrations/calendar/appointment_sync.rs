use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use futures::StreamExt;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};

use super::caldav::{fetch_existing_events_in_range, CaldavSession};
use super::commands::{employees_to_load, load_caldav_session, EMPLOYEE_FETCH_CONCURRENCY};
use super::events::{classify_event, parse_daylite_reference};
use super::slots::full_window;
use super::types::{PendingEvent, RawVEvent};
use crate::integrations::daylite::appointments::{
    AppointmentFields, AppointmentStore, DayliteAppointment, DayliteAppointments,
};
use crate::integrations::daylite::contacts::cached_contact_display_name;
use crate::integrations::daylite::projects::resolve_project_names;
use crate::integrations::local_store::{DayliteContactCacheEntry, LocalStore};
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::{
    record_failure_with, RequestFailure, TelemetryError,
};
use crate::integrations::telemetry::recorder::TelemetryRecorder;
use crate::integrations::telemetry::state::TelemetryState;

pub(super) struct WorkerEvents {
    pub(super) employee_reference: String,
    pub(super) events: Vec<RawVEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AppointmentAction {
    Create(AppointmentFields),
    Patch {
        reference: String,
        fields: AppointmentFields,
    },
    Delete {
        reference: String,
    },
}

struct ProjectDay {
    summary: String,
    worker_names: BTreeMap<String, String>,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

/// A project missing from `project_names` is named after the summary of its first event.
pub(super) fn desired_appointments(
    date: NaiveDate,
    workers: &[WorkerEvents],
    contacts: &[DayliteContactCacheEntry],
    project_names: &HashMap<String, String>,
) -> Vec<AppointmentFields> {
    let iso_date = date.format("%Y-%m-%d").to_string();
    let mut projects: BTreeMap<String, ProjectDay> = BTreeMap::new();

    for worker in workers {
        for event in &worker.events {
            let pending = classify_event(event);
            let Some(project_ref) = pending.project_ref.clone() else {
                continue;
            };
            if pending.date != iso_date {
                continue;
            }

            let (start, end) = event_window(date, &pending);
            let project = projects.entry(project_ref).or_insert_with(|| ProjectDay {
                summary: pending.summary.clone(),
                worker_names: BTreeMap::new(),
                start,
                end,
            });
            project.start = project.start.min(start);
            project.end = project.end.max(end);
            project
                .worker_names
                .entry(worker.employee_reference.clone())
                .or_insert_with(|| {
                    cached_contact_display_name(
                        contacts
                            .iter()
                            .find(|contact| contact.reference == worker.employee_reference),
                    )
                });
        }
    }

    projects
        .into_iter()
        .map(|(project_ref, project)| {
            let project_name = project_names
                .get(&project_ref)
                .filter(|name| !name.is_empty())
                .cloned()
                .unwrap_or(project.summary);
            let mut worker_names: Vec<String> = project.worker_names.into_values().collect();
            worker_names.sort_by_key(|name| (name.to_lowercase(), name.clone()));

            AppointmentFields {
                subject: format!("{project_name} ({})", worker_names.join(", ")),
                local_start: project.start,
                local_end: project.end,
                project_ref,
            }
        })
        .collect()
}

/// A missing or unreadable time falls back to the edge of the full assignment window.
/// An event that does not end after it starts runs until midnight, because Daylite rejects such an appointment.
fn event_window(date: NaiveDate, event: &PendingEvent) -> (NaiveDateTime, NaiveDateTime) {
    let (window_start, window_end) = full_window();
    let parse = |time: Option<&str>| NaiveTime::parse_from_str(time?, "%H:%M").ok();
    let start = date.and_time(parse(event.start_time.as_deref()).unwrap_or(window_start));
    let end = date.and_time(parse(event.end_time.as_deref()).unwrap_or(window_end));
    let midnight = (date + chrono::Duration::days(1)).and_time(NaiveTime::MIN);
    (start, if end > start { end } else { midnight })
}

/// Appointments are matched by their first linked project; of several for one project the lowest ID is kept.
pub(super) fn plan_appointment_actions(
    desired: Vec<AppointmentFields>,
    existing: Vec<DayliteAppointment>,
) -> Vec<AppointmentAction> {
    let mut actions = Vec::new();
    let mut by_project: BTreeMap<String, Vec<DayliteAppointment>> = BTreeMap::new();
    for appointment in existing {
        let project_ref = appointment.project_refs().next().map(str::to_string);
        match project_ref {
            Some(project_ref) => by_project.entry(project_ref).or_default().push(appointment),
            None => actions.push(delete(appointment)),
        }
    }

    for fields in desired {
        let mut candidates = by_project.remove(&fields.project_ref).unwrap_or_default();
        candidates.sort_by_key(DayliteAppointment::numeric_id);
        let mut candidates = candidates.into_iter();
        match candidates.next() {
            None => actions.push(AppointmentAction::Create(fields)),
            Some(kept) if kept.matches(&fields) => {}
            Some(kept) => actions.push(AppointmentAction::Patch {
                reference: kept.reference,
                fields,
            }),
        }
        actions.extend(candidates.map(delete));
    }

    actions.extend(by_project.into_values().flatten().map(delete));
    actions
}

fn delete(appointment: DayliteAppointment) -> AppointmentAction {
    AppointmentAction::Delete {
        reference: appointment.reference,
    }
}

/// A day is only written once every worker's calendar has been read, because a partial view would drop names.
/// One failed action does not stop the others, but fails the day.
pub(super) async fn reconcile_day<Names>(
    date: NaiveDate,
    workers: Result<Vec<WorkerEvents>, String>,
    contacts: &[DayliteContactCacheEntry],
    project_names: impl FnOnce(HashSet<String>) -> Names,
    appointments: &impl AppointmentStore,
    telemetry: Option<&TelemetryRecorder>,
) -> bool
where
    Names: Future<Output = HashMap<String, String>>,
{
    let workers = match workers {
        Ok(workers) => workers,
        Err(error) => {
            let failure = RequestFailure::new("CALENDAR_READ_FAILED", format!("{date}: {error}"));
            return sync_failed(telemetry, Integration::Zep, &failure);
        }
    };

    let references = workers
        .iter()
        .flat_map(|worker| &worker.events)
        .filter_map(|event| parse_daylite_reference(&event.description))
        .collect();
    let project_names = project_names(references).await;
    let desired = desired_appointments(date, &workers, contacts, &project_names);

    let existing = match appointments.search_day(date).await {
        Ok(existing) => existing,
        Err(error) => return sync_failed(telemetry, Integration::Daylite, &error),
    };

    let mut synced = true;
    for action in plan_appointment_actions(desired, existing) {
        let result = match &action {
            AppointmentAction::Create(fields) => appointments.create(fields).await.map(drop),
            AppointmentAction::Patch { reference, fields } => {
                appointments.patch(reference, fields).await
            }
            AppointmentAction::Delete { reference } => appointments.delete(reference).await,
        };
        if let Err(error) = result {
            synced = sync_failed(telemetry, Integration::Daylite, &error);
        }
    }

    synced
}

/// Always `false`, so a failing step can return its outcome directly.
fn sync_failed(
    telemetry: Option<&TelemetryRecorder>,
    integration: Integration,
    error: &dyn TelemetryError,
) -> bool {
    record_failure_with(
        telemetry,
        Operation::DayliteAppointmentSync,
        integration,
        error,
    );
    false
}

/// Daylite answers a burst of requests with 429, so a retry after an outage goes a few days at a time.
const RETRY_CONCURRENCY: usize = 2;

/// Matched by the frontend listener in `use-appointment-sync-failures.ts`.
const APPOINTMENT_SYNC_FAILURES_EVENT: &str = "daylite-appointment-sync-failures";

/// Serializes reconciliations per date, so two quick writes cannot both find no appointment and create one each.
#[derive(Default)]
pub(super) struct AppointmentSync {
    locks: Mutex<HashMap<NaiveDate, Arc<tokio::sync::Mutex<()>>>>,
    failed_dates: Mutex<BTreeSet<NaiveDate>>,
}

impl AppointmentSync {
    /// `notify` receives the sorted ISO dates still failing whenever that set changes.
    pub(super) async fn run<Fut, N>(
        &self,
        date: NaiveDate,
        reconcile: impl FnOnce() -> Fut,
        notify: &N,
    ) -> bool
    where
        Fut: Future<Output = bool>,
        N: Fn(Vec<String>),
    {
        let lock = self.lock_for(date);
        let _guard = lock.lock().await;
        let synced = reconcile().await;
        self.record_outcome(date, synced, notify);
        synced
    }

    /// Announces under the lock, so two outcomes finishing together cannot reach the frontend out of order.
    fn record_outcome(&self, date: NaiveDate, synced: bool, notify: &impl Fn(Vec<String>)) {
        let mut failed = self
            .failed_dates
            .lock()
            .expect("appointment sync failures poisoned");
        let changed = if synced {
            failed.remove(&date)
        } else {
            failed.insert(date)
        };
        if changed {
            notify(failed.iter().map(NaiveDate::to_string).collect());
        }
    }

    /// Reconciles every date failing when called, each under its date's lock.
    pub(super) async fn retry_failed<Fut, N>(
        &self,
        reconcile: impl Fn(NaiveDate) -> Fut,
        notify: &N,
    ) where
        Fut: Future<Output = bool>,
        N: Fn(Vec<String>),
    {
        let dates: Vec<NaiveDate> = self
            .failed_dates
            .lock()
            .expect("appointment sync failures poisoned")
            .iter()
            .copied()
            .collect();

        let reconcile = &reconcile;
        futures::stream::iter(dates)
            .map(|date| self.run(date, move || reconcile(date), notify))
            .buffer_unordered(RETRY_CONCURRENCY)
            .collect::<Vec<_>>()
            .await;
    }

    fn lock_for(&self, date: NaiveDate) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self.locks.lock().expect("appointment sync locks poisoned");
        Arc::clone(locks.entry(date).or_default())
    }
}

pub(super) fn appointment_sync() -> &'static AppointmentSync {
    static SYNC: OnceLock<AppointmentSync> = OnceLock::new();
    SYNC.get_or_init(AppointmentSync::default)
}

/// Runs after the command returned, so Daylite never adds latency to a planner action.
pub(super) fn schedule_appointment_sync(app: &tauri::AppHandle, dates: Vec<NaiveDate>) {
    for date in dates {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            appointment_sync()
                .run(
                    date,
                    || reconcile_date_in_app(&app, date),
                    &failure_notifier(&app),
                )
                .await;
        });
    }
}

pub(super) async fn retry_failed_in_app(app: &tauri::AppHandle) {
    appointment_sync()
        .retry_failed(
            |date| reconcile_date_in_app(app, date),
            &failure_notifier(app),
        )
        .await;
}

fn failure_notifier(app: &tauri::AppHandle) -> impl Fn(Vec<String>) + '_ {
    move |dates| {
        if let Err(error) = app.emit(APPOINTMENT_SYNC_FAILURES_EVENT, dates) {
            eprintln!("Failed to emit appointment sync failures: {error}");
        }
    }
}

/// Reads the configuration afresh, so a settings change applies to the next reconciliation.
pub(super) async fn reconcile_date_in_app(app: &tauri::AppHandle, date: NaiveDate) -> bool {
    let recorder = app
        .try_state::<TelemetryState>()
        .map(|state| Arc::clone(state.recorder()));
    let telemetry = recorder.as_deref();

    let store = match crate::integrations::local_store::load_local_store(app.clone()) {
        Ok(store) => store,
        Err(error) => return sync_failed(telemetry, Integration::LocalStore, &error),
    };
    let appointments = match DayliteAppointments::new(&store.api_endpoints.daylite_base_url, app) {
        Ok(appointments) => appointments,
        Err(error) => return sync_failed(telemetry, Integration::Daylite, &error),
    };
    let session = match load_caldav_session(app, &store) {
        Ok(session) => session,
        Err(error) => return sync_failed(telemetry, Integration::Zep, &error),
    };

    reconcile_day(
        date,
        fetch_worker_events(&store, &session, date).await,
        &store.daylite_cache.contacts,
        |references| resolve_project_names(app.clone(), references),
        &appointments,
        telemetry,
    )
    .await
}

/// Reads the planning contacts the grid lists, Monteur and Test alike, so a former contact's leftover calendar cannot fail the day.
/// A calendar that does not exist cannot hold assignments, so it counts as empty.
async fn fetch_worker_events(
    store: &LocalStore,
    session: &CaldavSession,
    date: NaiveDate,
) -> Result<Vec<WorkerEvents>, String> {
    let next_day = date + chrono::Duration::days(1);
    let planning_contacts: Vec<String> = store
        .daylite_cache
        .contacts
        .iter()
        .map(|contact| contact.reference.clone())
        .collect();
    let fetches: Vec<_> = employees_to_load(store, Some(&planning_contacts))
        .map(|setting| {
            let employee_reference = setting.daylite_contact_reference.clone();
            let calendar_url = setting.zep_primary_calendar.clone().unwrap_or_default();
            async move {
                let events = fetch_existing_events_in_range(session, &calendar_url, date, next_day)
                    .await
                    .map_err(|error| format!("{employee_reference}: {error}"))?;
                Ok(events.map(|events| WorkerEvents {
                    employee_reference,
                    events,
                }))
            }
        })
        .collect();

    let fetched: Vec<Option<WorkerEvents>> = futures::stream::iter(fetches)
        .buffered(EMPLOYEE_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<_, String>>()?;

    Ok(fetched.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::daylite::appointments::DayliteProjectLink;
    use crate::integrations::daylite::shared::{DayliteApiError, DayliteApiErrorCode};
    use crate::integrations::local_store::{DayliteCache, EmployeeSetting};
    use crate::integrations::telemetry::recorder::TelemetryCommand;
    use crate::integrations::telemetry::test_support::{enabled_recorder, recorded};
    use crate::integrations::test_http::TestServer;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    const DATE: &str = "2026-11-02";
    const NORD: &str = "/v1/projects/42";
    const SUED: &str = "/v1/projects/43";
    const MAX: &str = "/v1/contacts/1";
    const ANNA: &str = "/v1/contacts/2";
    const THIRD: &str = "/v1/contacts/3";

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()
    }

    fn next_day() -> NaiveDate {
        day() + chrono::Duration::days(1)
    }

    fn at(time: &str) -> NaiveDateTime {
        day().and_time(NaiveTime::parse_from_str(time, "%H:%M").unwrap())
    }

    /// An empty time stands for an event without one.
    fn assignment(project_ref: &str, summary: &str, start: &str, end: &str) -> RawVEvent {
        let time = |value: &str| Some(value.to_string()).filter(|value| !value.is_empty());
        RawVEvent {
            uid: format!("{project_ref}-{start}"),
            summary: summary.to_string(),
            description: format!("daylite:{project_ref}"),
            dtstart: DATE.to_string(),
            start_time: time(start),
            end_time: time(end),
            ..Default::default()
        }
    }

    fn on_nord() -> RawVEvent {
        assignment(NORD, "Projekt Nord", "08:00", "16:00")
    }

    fn worker(employee_reference: &str, events: Vec<RawVEvent>) -> WorkerEvents {
        WorkerEvents {
            employee_reference: employee_reference.to_string(),
            events,
        }
    }

    fn contact(
        reference: &str,
        full_name: Option<&str>,
        nickname: Option<&str>,
    ) -> DayliteContactCacheEntry {
        DayliteContactCacheEntry {
            reference: reference.to_string(),
            full_name: full_name.map(str::to_string),
            nickname: nickname.map(str::to_string),
            ..Default::default()
        }
    }

    fn contacts() -> Vec<DayliteContactCacheEntry> {
        vec![
            contact(MAX, Some("Max Müller"), None),
            contact(ANNA, Some("Anna Schmidt"), None),
        ]
    }

    fn names() -> HashMap<String, String> {
        HashMap::from([
            (NORD.to_string(), "Projekt Nord".to_string()),
            (SUED.to_string(), "Projekt Süd".to_string()),
        ])
    }

    async fn known_names(_: HashSet<String>) -> HashMap<String, String> {
        names()
    }

    fn desired(workers: &[WorkerEvents]) -> Vec<AppointmentFields> {
        desired_appointments(day(), workers, &contacts(), &names())
    }

    fn fields(project_ref: &str, subject: &str, start: &str, end: &str) -> AppointmentFields {
        AppointmentFields {
            project_ref: project_ref.to_string(),
            subject: subject.to_string(),
            local_start: at(start),
            local_end: at(end),
        }
    }

    fn nord() -> AppointmentFields {
        fields(NORD, "Projekt Nord (Max Müller)", "08:00", "12:00")
    }

    fn link(project: &str) -> DayliteProjectLink {
        DayliteProjectLink {
            project: project.to_string(),
        }
    }

    fn existing(reference: &str, matching: &AppointmentFields) -> DayliteAppointment {
        let local = |time: NaiveDateTime| Some(time.format("%Y-%m-%dT%H:%M:%S").to_string());
        DayliteAppointment {
            reference: reference.to_string(),
            subject: matching.subject.clone(),
            timezone: Some("floating".to_string()),
            local_start: local(matching.local_start),
            local_end: local(matching.local_end),
            appointment_type: Some("appointment".to_string()),
            status: Some("confirmed".to_string()),
            show_as: Some("free".to_string()),
            invitations: Some(vec![]),
            projects: Some(vec![link(&matching.project_ref)]),
        }
    }

    fn deleted(reference: &str) -> AppointmentAction {
        AppointmentAction::Delete {
            reference: reference.to_string(),
        }
    }

    #[test]
    fn builds_one_appointment_per_project() {
        let workers = [
            worker(
                MAX,
                vec![
                    assignment(NORD, "Projekt Nord", "08:00", "12:00"),
                    assignment(SUED, "Projekt Süd", "12:00", "16:00"),
                ],
            ),
            worker(ANNA, vec![on_nord()]),
        ];

        let projects: Vec<String> = desired(&workers)
            .into_iter()
            .map(|fields| fields.project_ref)
            .collect();

        assert_eq!(projects, [NORD, SUED]);
    }

    #[test]
    fn titles_name_the_project_and_each_worker_once() {
        let everyone_on_nord = || {
            [MAX, ANNA, THIRD]
                .map(|reference| worker(reference, vec![on_nord()]))
                .into()
        };
        let cases: Vec<(&str, Vec<DayliteContactCacheEntry>, Vec<WorkerEvents>, &str)> = vec![
            (
                "two workers",
                contacts(),
                vec![worker(MAX, vec![on_nord()]), worker(ANNA, vec![on_nord()])],
                "Projekt Nord (Anna Schmidt, Max Müller)",
            ),
            (
                "a worker with two assignments is listed once",
                contacts(),
                vec![worker(
                    MAX,
                    vec![
                        on_nord(),
                        assignment(NORD, "Projekt Nord", "12:00", "16:00"),
                    ],
                )],
                "Projekt Nord (Max Müller)",
            ),
            (
                "names sort ignoring case",
                vec![
                    contact(MAX, Some("bernd Becker"), None),
                    contact(ANNA, Some("Anna Schmidt"), None),
                    contact(THIRD, Some("Carla Conrad"), None),
                ],
                everyone_on_nord(),
                "Projekt Nord (Anna Schmidt, bernd Becker, Carla Conrad)",
            ),
            (
                "nickname, then full name, then placeholder",
                vec![
                    contact(MAX, Some("Max Müller"), Some("Maxi")),
                    contact(ANNA, Some("Anna Schmidt"), None),
                    contact(THIRD, None, None),
                ],
                everyone_on_nord(),
                "Projekt Nord (Anna Schmidt, Maxi, Unbenannter Kontakt)",
            ),
            (
                "an unresolved project is named after its event",
                contacts(),
                vec![worker(
                    MAX,
                    vec![assignment(
                        "/v1/projects/99",
                        "Projekt Ost",
                        "08:00",
                        "16:00",
                    )],
                )],
                "Projekt Ost (Max Müller)",
            ),
        ];

        for (label, contacts, workers, expected) in cases {
            let subjects: Vec<String> = desired_appointments(day(), &workers, &contacts, &names())
                .into_iter()
                .map(|fields| fields.subject)
                .collect();

            assert_eq!(subjects, [expected], "case: {label}");
        }
    }

    #[test]
    fn times_span_the_projects_assignments_of_the_day() {
        let cases = [
            (
                "earliest start and latest end across workers",
                vec![
                    worker(
                        MAX,
                        vec![assignment(NORD, "Projekt Nord", "10:00", "12:00")],
                    ),
                    worker(
                        ANNA,
                        vec![assignment(NORD, "Projekt Nord", "08:00", "09:00")],
                    ),
                ],
                at("08:00"),
                at("12:00"),
            ),
            (
                "an event without times counts as the full window",
                vec![worker(MAX, vec![assignment(NORD, "Projekt Nord", "", "")])],
                at("08:00"),
                at("16:00"),
            ),
            (
                "an event that does not end after it starts runs until midnight",
                vec![worker(
                    MAX,
                    vec![assignment(NORD, "Projekt Nord", "17:00", "")],
                )],
                at("17:00"),
                next_day().and_time(NaiveTime::MIN),
            ),
        ];

        for (label, workers, start, end) in cases {
            let appointment = &desired(&workers)[0];

            assert_eq!(
                (appointment.local_start, appointment.local_end),
                (start, end),
                "case: {label}"
            );
        }
    }

    #[test]
    fn ignores_bare_events_absences_and_other_days() {
        let workers = [worker(
            MAX,
            vec![
                RawVEvent {
                    description: "Bitte Auto abholen".to_string(),
                    ..assignment(NORD, "Werkstatt", "08:00", "09:00")
                },
                RawVEvent {
                    description: String::new(),
                    ..assignment(NORD, "Urlaub", "", "")
                },
                RawVEvent {
                    dtstart: "2026-11-03".to_string(),
                    ..on_nord()
                },
            ],
        )];

        assert!(desired(&workers).is_empty());
    }

    #[test]
    fn creates_the_appointment_of_a_project_without_one() {
        let actions = plan_appointment_actions(vec![nord()], vec![]);

        assert_eq!(actions, [AppointmentAction::Create(nord())]);
    }

    #[test]
    fn deletes_an_appointment_whose_project_has_no_assignments() {
        let actions =
            plan_appointment_actions(vec![], vec![existing("/v1/appointments/5000", &nord())]);

        assert_eq!(actions, [deleted("/v1/appointments/5000")]);
    }

    #[test]
    fn leaves_a_matching_appointment_alone() {
        let actions = plan_appointment_actions(
            vec![nord()],
            vec![existing("/v1/appointments/5000", &nord())],
        );

        assert!(actions.is_empty());
    }

    #[test]
    fn patches_an_appointment_that_differs_in_an_owned_field() {
        let matching = existing("/v1/appointments/5000", &nord());
        let cases = [
            (
                DayliteAppointment {
                    subject: "Projekt Nord (Anna Schmidt, Max Müller)".to_string(),
                    ..matching.clone()
                },
                "subject",
            ),
            (
                DayliteAppointment {
                    local_end: Some("2026-11-02T16:00:00".to_string()),
                    ..matching.clone()
                },
                "end time",
            ),
            (
                DayliteAppointment {
                    timezone: Some("Europe/Berlin".to_string()),
                    ..matching.clone()
                },
                "time zone",
            ),
            (
                DayliteAppointment {
                    status: Some("tentative".to_string()),
                    ..matching.clone()
                },
                "status",
            ),
            (
                DayliteAppointment {
                    show_as: Some("busy".to_string()),
                    ..matching.clone()
                },
                "show as",
            ),
            (
                DayliteAppointment {
                    appointment_type: Some("meeting".to_string()),
                    ..matching.clone()
                },
                "type",
            ),
            (
                DayliteAppointment {
                    invitations: Some(vec![serde_json::json!({ "invitee": "/v1/users/1000" })]),
                    ..matching.clone()
                },
                "invitees",
            ),
            (
                DayliteAppointment {
                    projects: Some(vec![link(NORD), link(SUED)]),
                    ..matching.clone()
                },
                "projects",
            ),
        ];

        for (appointment, label) in cases {
            let actions = plan_appointment_actions(vec![nord()], vec![appointment]);

            assert_eq!(
                actions,
                [AppointmentAction::Patch {
                    reference: "/v1/appointments/5000".to_string(),
                    fields: nord(),
                }],
                "case: {label}"
            );
        }
    }

    #[test]
    fn keeps_the_lowest_id_of_duplicate_appointments_and_deletes_the_rest() {
        let actions = plan_appointment_actions(
            vec![nord()],
            vec![
                existing("/v1/appointments/1000", &nord()),
                DayliteAppointment {
                    status: Some("tentative".to_string()),
                    ..existing("/v1/appointments/900", &nord())
                },
                existing("/v1/appointments/1100", &nord()),
            ],
        );

        assert_eq!(
            actions,
            [
                AppointmentAction::Patch {
                    reference: "/v1/appointments/900".to_string(),
                    fields: nord(),
                },
                deleted("/v1/appointments/1000"),
                deleted("/v1/appointments/1100"),
            ]
        );
    }

    #[test]
    fn deletes_an_appointment_linked_to_no_project() {
        let actions = plan_appointment_actions(
            vec![nord()],
            vec![
                existing("/v1/appointments/5000", &nord()),
                DayliteAppointment {
                    projects: None,
                    ..existing("/v1/appointments/5001", &nord())
                },
            ],
        );

        assert_eq!(actions, [deleted("/v1/appointments/5001")]);
    }

    #[derive(Default)]
    struct StubAppointments {
        existing: Vec<DayliteAppointment>,
        search_fails: bool,
        /// Project references whose create fails, and appointment references whose patch or delete fails.
        failing: Vec<String>,
        requests: Mutex<Vec<String>>,
    }

    impl StubAppointments {
        fn respond<T>(&self, request: String, fails: bool, value: T) -> Result<T, DayliteApiError> {
            self.requests.lock().unwrap().push(request);
            if fails {
                return Err(DayliteApiError {
                    code: DayliteApiErrorCode::ServerError,
                    http_status: Some(503),
                    user_message: "Daylite ist aktuell nicht erreichbar.".to_string(),
                    technical_message: "status=503".to_string(),
                });
            }
            Ok(value)
        }

        fn fails(&self, key: &str) -> bool {
            self.failing.iter().any(|failing| failing == key)
        }

        fn requests(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl AppointmentStore for StubAppointments {
        async fn search_day(
            &self,
            date: NaiveDate,
        ) -> Result<Vec<DayliteAppointment>, DayliteApiError> {
            self.respond(
                format!("search {date}"),
                self.search_fails,
                self.existing.clone(),
            )
        }

        async fn create(&self, fields: &AppointmentFields) -> Result<String, DayliteApiError> {
            self.respond(
                format!("create {}", fields.project_ref),
                self.fails(&fields.project_ref),
                "/v1/appointments/7000".to_string(),
            )
        }

        async fn patch(
            &self,
            reference: &str,
            _: &AppointmentFields,
        ) -> Result<(), DayliteApiError> {
            self.respond(format!("patch {reference}"), self.fails(reference), ())
        }

        async fn delete(&self, reference: &str) -> Result<(), DayliteApiError> {
            self.respond(format!("delete {reference}"), self.fails(reference), ())
        }
    }

    fn two_projects() -> Vec<WorkerEvents> {
        vec![
            worker(
                MAX,
                vec![assignment(NORD, "Projekt Nord", "08:00", "12:00")],
            ),
            worker(
                ANNA,
                vec![assignment(SUED, "Projekt Süd", "08:00", "16:00")],
            ),
        ]
    }

    fn unreadable_calendar() -> Result<Vec<WorkerEvents>, String> {
        Err("CalDAV-Server antwortete mit HTTP 500".to_string())
    }

    async fn reconcile(
        workers: Result<Vec<WorkerEvents>, String>,
        appointments: &StubAppointments,
        telemetry: Option<&TelemetryRecorder>,
    ) -> bool {
        reconcile_day(
            day(),
            workers,
            &contacts(),
            known_names,
            appointments,
            telemetry,
        )
        .await
    }

    fn stale(reference: &str) -> DayliteAppointment {
        existing(
            reference,
            &fields(
                "/v1/projects/44",
                "Projekt West (Max Müller)",
                "08:00",
                "16:00",
            ),
        )
    }

    /// Each recorded error as `<operation> <integration>`.
    fn sync_errors(receiver: &mut tokio::sync::mpsc::Receiver<TelemetryCommand>) -> Vec<String> {
        recorded(receiver)
            .into_iter()
            .filter(|event| event.name() == "error_occurred")
            .map(|event| {
                let property = |key: &str| event.properties()[key].as_str().unwrap().to_string();
                format!("{} {}", property("operation"), property("integration"))
            })
            .collect()
    }

    #[tokio::test]
    async fn an_unreadable_worker_calendar_sends_no_daylite_request() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments::default();

        let synced = reconcile(unreadable_calendar(), &appointments, Some(&recorder)).await;

        assert!(!synced);
        assert!(appointments.requests().is_empty());
        assert_eq!(sync_errors(&mut receiver), ["daylite_appointment_sync zep"]);
    }

    #[tokio::test]
    async fn a_failed_create_still_applies_the_other_projects_actions() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![stale("/v1/appointments/5000")],
            failing: vec![NORD.to_string()],
            ..Default::default()
        };

        let synced = reconcile(Ok(two_projects()), &appointments, Some(&recorder)).await;

        assert!(!synced);
        assert_eq!(
            appointments.requests(),
            [
                "search 2026-11-02",
                "create /v1/projects/42",
                "create /v1/projects/43",
                "delete /v1/appointments/5000",
            ]
        );
        assert_eq!(
            sync_errors(&mut receiver),
            ["daylite_appointment_sync daylite"]
        );
    }

    #[tokio::test]
    async fn every_daylite_failure_is_recorded_under_the_sync_operation() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![
                DayliteAppointment {
                    status: Some("tentative".to_string()),
                    ..existing("/v1/appointments/5000", &nord())
                },
                stale("/v1/appointments/5001"),
            ],
            failing: vec![
                "/v1/appointments/5000".to_string(),
                "/v1/appointments/5001".to_string(),
            ],
            ..Default::default()
        };

        let synced = reconcile(Ok(two_projects()), &appointments, Some(&recorder)).await;

        assert!(!synced);
        assert_eq!(
            sync_errors(&mut receiver),
            [
                "daylite_appointment_sync daylite",
                "daylite_appointment_sync daylite"
            ]
        );
    }

    #[tokio::test]
    async fn a_failed_search_changes_nothing() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments {
            search_fails: true,
            ..Default::default()
        };

        let synced = reconcile(Ok(two_projects()), &appointments, Some(&recorder)).await;

        assert!(!synced);
        assert_eq!(appointments.requests(), ["search 2026-11-02"]);
        assert_eq!(
            sync_errors(&mut receiver),
            ["daylite_appointment_sync daylite"]
        );
    }

    #[tokio::test]
    async fn a_day_that_converges_reports_success_without_errors() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![existing("/v1/appointments/5000", &nord())],
            ..Default::default()
        };

        let synced = reconcile(Ok(two_projects()), &appointments, Some(&recorder)).await;

        assert!(synced);
        assert_eq!(
            appointments.requests(),
            ["search 2026-11-02", "create /v1/projects/43"]
        );
        assert!(sync_errors(&mut receiver).is_empty());
    }

    fn ignore(_: Vec<String>) {}

    #[derive(Default)]
    struct Notifications(Mutex<Vec<Vec<String>>>);

    impl Notifications {
        fn notify(&self) -> impl Fn(Vec<String>) + '_ {
            |dates| self.0.lock().unwrap().push(dates)
        }

        fn received(&self) -> Vec<Vec<String>> {
            self.0.lock().unwrap().clone()
        }
    }

    async fn settle(
        sync: &AppointmentSync,
        date: NaiveDate,
        synced: bool,
        notify: &impl Fn(Vec<String>),
    ) {
        sync.run(date, || async move { synced }, notify).await;
    }

    async fn failing(dates: &[NaiveDate]) -> AppointmentSync {
        let sync = AppointmentSync::default();
        for date in dates {
            settle(&sync, *date, false, &ignore).await;
        }
        sync
    }

    #[tokio::test]
    async fn reconciliations_of_one_date_run_one_after_the_other() {
        let sync = AppointmentSync::default();
        let log = Mutex::new(Vec::new());
        let (release, released) = tokio::sync::oneshot::channel::<()>();

        tokio::join!(
            sync.run(
                day(),
                || async {
                    log.lock().unwrap().push("first started");
                    released.await.unwrap();
                    log.lock().unwrap().push("first finished");
                    true
                },
                &ignore,
            ),
            sync.run(
                day(),
                || async {
                    log.lock().unwrap().push("second started");
                    true
                },
                &ignore,
            ),
            sync.run(
                next_day(),
                || async {
                    log.lock().unwrap().push("other day started");
                    release.send(()).unwrap();
                    true
                },
                &ignore,
            ),
        );

        assert_eq!(
            *log.lock().unwrap(),
            [
                "first started",
                "other day started",
                "first finished",
                "second started"
            ]
        );
    }

    #[tokio::test]
    async fn the_failed_dates_follow_each_outcome() {
        let dates =
            |dates: &[&str]| -> Vec<String> { dates.iter().map(|date| date.to_string()).collect() };
        let cases = vec![
            (
                "failures are listed in date order",
                vec![(next_day(), false), (day(), false)],
                vec![dates(&["2026-11-03"]), dates(&["2026-11-02", "2026-11-03"])],
            ),
            (
                "a success clears its date",
                vec![(day(), false), (day(), true)],
                vec![dates(&["2026-11-02"]), vec![]],
            ),
            (
                "an unchanged set is not announced again",
                vec![(day(), true), (day(), false), (day(), false)],
                vec![dates(&["2026-11-02"])],
            ),
        ];

        for (label, outcomes, expected) in cases {
            let sync = AppointmentSync::default();
            let notifications = Notifications::default();
            for (date, synced) in outcomes {
                settle(&sync, date, synced, &notifications.notify()).await;
            }

            assert_eq!(notifications.received(), expected, "case: {label}");
        }
    }

    #[tokio::test]
    async fn a_day_with_an_unreadable_worker_calendar_counts_as_failed() {
        let sync = AppointmentSync::default();
        let notifications = Notifications::default();
        let appointments = StubAppointments::default();

        sync.run(
            day(),
            || reconcile(unreadable_calendar(), &appointments, None),
            &notifications.notify(),
        )
        .await;

        assert_eq!(notifications.received(), [vec!["2026-11-02".to_string()]]);
    }

    #[tokio::test]
    async fn a_change_is_announced_before_another_outcome_can_be_recorded() {
        let sync = AppointmentSync::default();
        let registry_held = AtomicBool::new(false);

        settle(&sync, day(), false, &|_| {
            registry_held.store(sync.failed_dates.try_lock().is_err(), Ordering::SeqCst)
        })
        .await;

        assert!(registry_held.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn a_retry_reconciles_every_failed_date_before_it_resolves() {
        let sync = failing(&[day(), next_day()]).await;
        let notifications = Notifications::default();
        let finished = Mutex::new(Vec::new());

        sync.retry_failed(
            |date| {
                let finished = &finished;
                async move {
                    tokio::task::yield_now().await;
                    finished.lock().unwrap().push(date);
                    true
                }
            },
            &notifications.notify(),
        )
        .await;

        let mut finished = finished.lock().unwrap().clone();
        finished.sort();
        assert_eq!(finished, [day(), next_day()]);
        assert_eq!(notifications.received().last(), Some(&vec![]));
    }

    #[tokio::test]
    async fn a_retry_keeps_only_the_dates_that_fail_again() {
        let sync = failing(&[day(), next_day()]).await;
        let notifications = Notifications::default();

        sync.retry_failed(|date| async move { date == day() }, &notifications.notify())
            .await;

        assert_eq!(notifications.received(), [vec!["2026-11-03".to_string()]]);
    }

    #[tokio::test]
    async fn a_retry_waits_for_a_write_reconciling_the_same_date() {
        let sync = failing(&[day()]).await;
        let log = Mutex::new(Vec::new());
        let (release, released) = tokio::sync::oneshot::channel::<()>();

        tokio::join!(
            sync.run(
                day(),
                || async {
                    log.lock().unwrap().push("write started");
                    released.await.unwrap();
                    log.lock().unwrap().push("write finished");
                    true
                },
                &ignore,
            ),
            async {
                sync.retry_failed(
                    |_| async {
                        log.lock().unwrap().push("retry started");
                        true
                    },
                    &ignore,
                )
                .await;
                log.lock().unwrap().push("retry resolved");
            },
            async {
                tokio::task::yield_now().await;
                release.send(()).unwrap();
            },
        );

        assert_eq!(
            *log.lock().unwrap(),
            [
                "write started",
                "write finished",
                "retry started",
                "retry resolved"
            ]
        );
    }

    #[tokio::test]
    async fn a_retry_reconciles_at_most_two_dates_at_once() {
        let dates: Vec<NaiveDate> = (0..5)
            .map(|offset| day() + chrono::Duration::days(offset))
            .collect();
        let sync = failing(&dates).await;
        let in_flight = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);

        sync.retry_failed(
            |_| async {
                let running = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(running, Ordering::SeqCst);
                tokio::task::yield_now().await;
                in_flight.fetch_sub(1, Ordering::SeqCst);
                true
            },
            &ignore,
        )
        .await;

        assert_eq!(peak.load(Ordering::SeqCst), 2);
    }

    const ONE_ASSIGNMENT_REPORT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/cal-max/uid-1.ics</d:href>
    <d:propstat>
      <d:prop>
        <c:calendar-data>BEGIN:VCALENDAR
BEGIN:VEVENT
UID:uid-1
SUMMARY:Projekt Nord
DESCRIPTION:daylite:/v1/projects/42
DTSTART:20261102T080000
DTEND:20261102T160000
END:VEVENT
END:VCALENDAR
</c:calendar-data>
      </d:prop>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    /// Every listed calendar is configured, but only the planning contacts are in the contact cache.
    fn calendar_store(
        base_url: &str,
        calendars: &[(&str, &str)],
        planning_contacts: &[&str],
    ) -> LocalStore {
        LocalStore {
            employee_settings: calendars
                .iter()
                .map(|(reference, calendar)| EmployeeSetting {
                    daylite_contact_reference: reference.to_string(),
                    zep_primary_calendar: Some(format!("{base_url}/{calendar}/")),
                    ..Default::default()
                })
                .collect(),
            daylite_cache: DayliteCache {
                last_synced_at: None,
                contacts: planning_contacts
                    .iter()
                    .map(|reference| contact(reference, None, None))
                    .collect(),
            },
            ..Default::default()
        }
    }

    fn calendar_session(base_url: &str) -> CaldavSession {
        CaldavSession {
            client: tauri_plugin_http::reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            username: "user".to_string(),
            password: "pass".to_string(),
            base_url: base_url.to_string(),
            absence_urls: vec![],
            telemetry: None,
        }
    }

    async fn read_max_and_third(third_status: u16) -> Result<Vec<WorkerEvents>, String> {
        let server = TestServer::spawn_with_bodies(vec![
            ("REPORT", "/cal-max/", 207, ONE_ASSIGNMENT_REPORT),
            ("REPORT", "/cal-third/", third_status, ""),
        ])
        .await;
        let store = calendar_store(
            &server.base_url,
            &[(MAX, "cal-max"), (THIRD, "cal-third")],
            &[MAX, THIRD],
        );

        fetch_worker_events(&store, &calendar_session(&server.base_url), day()).await
    }

    #[tokio::test]
    async fn only_the_planning_contacts_calendars_are_read() {
        let former = "/v1/contacts/4";
        let server = TestServer::spawn_with_bodies(vec![
            ("REPORT", "/cal-max/", 207, ONE_ASSIGNMENT_REPORT),
            ("REPORT", "/cal-tester/", 207, ONE_ASSIGNMENT_REPORT),
            ("REPORT", "/cal-former/", 503, ""),
        ])
        .await;
        let mut store = calendar_store(
            &server.base_url,
            &[
                (MAX, "cal-max"),
                (ANNA, "cal-tester"),
                (former, "cal-former"),
            ],
            &[MAX, ANNA],
        );
        store.daylite_cache.contacts[0].category = Some("Monteur".to_string());
        store.daylite_cache.contacts[1].category = Some("Test".to_string());

        let workers = fetch_worker_events(&store, &calendar_session(&server.base_url), day())
            .await
            .expect("a former contact's calendar should not be read");

        let read: Vec<&str> = workers
            .iter()
            .map(|worker| worker.employee_reference.as_str())
            .collect();
        assert_eq!(read, [MAX, ANNA]);
        assert!(server
            .requests()
            .iter()
            .all(|(_, path)| !path.starts_with("/cal-former/")));
    }

    #[tokio::test]
    async fn a_calendar_that_does_not_exist_holds_no_assignments() {
        let workers = read_max_and_third(404)
            .await
            .expect("a missing calendar should not fail the day");

        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].employee_reference, MAX);
        assert_eq!(workers[0].events.len(), 1);
    }

    #[tokio::test]
    async fn an_unreadable_calendar_still_fails_the_day() {
        let error = read_max_and_third(503)
            .await
            .err()
            .expect("a calendar that answers with a server error should fail the day");

        assert_eq!(
            error,
            "/v1/contacts/3: CalDAV-Server antwortete mit HTTP 503"
        );
    }
}
