use chrono::{NaiveDate, NaiveTime};
use futures::StreamExt;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};

use super::caldav::{fetch_events_in_range, CaldavSession};
use super::commands::{employees_to_load, load_caldav_session, EMPLOYEE_FETCH_CONCURRENCY};
use super::events::{classify_event, parse_daylite_reference};
use super::slots::full_window;
use super::types::{PendingEvent, RawVEvent};
use crate::integrations::daylite::appointments::{
    AppointmentFields, DayliteAppointment, DayliteAppointments,
};
use crate::integrations::daylite::contacts::cached_contact_display_name;
use crate::integrations::daylite::projects::resolve_project_names;
use crate::integrations::daylite::shared::DayliteApiError;
use crate::integrations::local_store::{DayliteContactCacheEntry, LocalStore};
use crate::integrations::telemetry::events::{Integration, Operation, TelemetryEvent};
use crate::integrations::telemetry::observe::{RequestFailure, TelemetryError};
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
    start: NaiveTime,
    end: NaiveTime,
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

            let (start, end) = event_window(&pending);
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
                local_start: date.and_time(project.start),
                local_end: date.and_time(project.end),
                project_ref,
            }
        })
        .collect()
}

/// A missing or unreadable time falls back to the edge of the full assignment window.
fn event_window(event: &PendingEvent) -> (NaiveTime, NaiveTime) {
    let (window_start, window_end) = full_window();
    let parse = |time: Option<&str>| NaiveTime::parse_from_str(time?, "%H:%M").ok();
    (
        parse(event.start_time.as_deref()).unwrap_or(window_start),
        parse(event.end_time.as_deref()).unwrap_or(window_end),
    )
}

/// Appointments are matched by their first linked project; of several for one project the lowest ID is kept.
pub(super) fn plan_appointment_actions(
    desired: Vec<AppointmentFields>,
    existing: Vec<DayliteAppointment>,
) -> Vec<AppointmentAction> {
    let mut actions = Vec::new();
    let mut by_project: BTreeMap<String, Vec<DayliteAppointment>> = BTreeMap::new();
    for appointment in existing {
        match appointment.project_refs.first() {
            Some(project_ref) => by_project
                .entry(project_ref.clone())
                .or_default()
                .push(appointment),
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

pub(super) trait DaySources {
    /// Fails when any configured primary calendar could not be read.
    fn worker_events(
        &self,
        date: NaiveDate,
    ) -> impl Future<Output = Result<Vec<WorkerEvents>, String>> + Send;

    fn contacts(&self) -> &[DayliteContactCacheEntry];

    /// A reference that cannot be resolved is left out.
    fn project_names(
        &self,
        references: HashSet<String>,
    ) -> impl Future<Output = HashMap<String, String>> + Send;
}

pub(super) trait AppointmentStore {
    fn search_day(
        &self,
        date: NaiveDate,
    ) -> impl Future<Output = Result<Vec<DayliteAppointment>, DayliteApiError>> + Send;

    fn create(
        &self,
        fields: &AppointmentFields,
    ) -> impl Future<Output = Result<String, DayliteApiError>> + Send;

    fn patch(
        &self,
        reference: &str,
        fields: &AppointmentFields,
    ) -> impl Future<Output = Result<(), DayliteApiError>> + Send;

    fn delete(&self, reference: &str) -> impl Future<Output = Result<(), DayliteApiError>> + Send;
}

/// A day is only written once every worker's calendar has been read, because a partial view would drop names.
/// One failed action does not stop the others, but fails the day.
pub(super) async fn reconcile_day(
    date: NaiveDate,
    sources: &impl DaySources,
    appointments: &impl AppointmentStore,
    telemetry: Option<&TelemetryRecorder>,
) -> bool {
    let workers = match sources.worker_events(date).await {
        Ok(workers) => workers,
        Err(error) => {
            record_failure(
                telemetry,
                Integration::Zep,
                &RequestFailure::new("CALENDAR_READ_FAILED", format!("{date}: {error}")),
            );
            return false;
        }
    };

    let references = workers
        .iter()
        .flat_map(|worker| &worker.events)
        .filter_map(|event| parse_daylite_reference(&event.description))
        .collect();
    let project_names = sources.project_names(references).await;
    let desired = desired_appointments(date, &workers, sources.contacts(), &project_names);

    let existing = match appointments.search_day(date).await {
        Ok(existing) => existing,
        Err(error) => {
            record_failure(telemetry, Integration::Daylite, &error);
            return false;
        }
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
            record_failure(telemetry, Integration::Daylite, &error);
            synced = false;
        }
    }

    synced
}

fn record_failure(
    telemetry: Option<&TelemetryRecorder>,
    integration: Integration,
    error: &impl TelemetryError,
) {
    let Some(recorder) = telemetry else {
        return;
    };

    recorder.record(
        TelemetryEvent::error_occurred(
            Operation::DayliteAppointmentSync,
            integration,
            error.error_code(),
            error.technical_message(),
        )
        .with_http_status(error.http_status()),
    );
}

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

    fn record_outcome(&self, date: NaiveDate, synced: bool, notify: &impl Fn(Vec<String>)) {
        let still_failing: Vec<String> = {
            let mut failed = self
                .failed_dates
                .lock()
                .expect("appointment sync failures poisoned");
            let changed = if synced {
                failed.remove(&date)
            } else {
                failed.insert(date)
            };
            if !changed {
                return;
            }
            failed.iter().map(NaiveDate::to_string).collect()
        };

        notify(still_failing);
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
        futures::future::join_all(
            dates
                .into_iter()
                .map(|date| self.run(date, move || reconcile(date), notify)),
        )
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
        Err(error) => {
            record_failure(telemetry, Integration::LocalStore, &error);
            return false;
        }
    };
    let appointments = match DayliteAppointments::new(&store.api_endpoints.daylite_base_url, app) {
        Ok(appointments) => appointments,
        Err(error) => {
            record_failure(telemetry, Integration::Daylite, &error);
            return false;
        }
    };
    let session = match load_caldav_session(app, &store) {
        Ok(session) => session,
        Err(error) => {
            record_failure(telemetry, Integration::Zep, &error);
            return false;
        }
    };

    let sources = AppSources {
        app: app.clone(),
        store,
        session,
    };
    reconcile_day(date, &sources, &appointments, telemetry).await
}

struct AppSources {
    app: tauri::AppHandle,
    store: LocalStore,
    session: CaldavSession,
}

impl DaySources for AppSources {
    async fn worker_events(&self, date: NaiveDate) -> Result<Vec<WorkerEvents>, String> {
        let next_day = date + chrono::Duration::days(1);
        let fetches: Vec<_> = employees_to_load(&self.store, None)
            .map(|setting| {
                let employee_reference = setting.daylite_contact_reference.clone();
                let calendar_url = setting.zep_primary_calendar.clone().unwrap_or_default();
                async move {
                    let events =
                        fetch_events_in_range(&self.session, &calendar_url, date, next_day)
                            .await
                            .map_err(|error| format!("{employee_reference}: {error}"))?;
                    Ok(WorkerEvents {
                        employee_reference,
                        events,
                    })
                }
            })
            .collect();

        futures::stream::iter(fetches)
            .buffered(EMPLOYEE_FETCH_CONCURRENCY)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
    }

    fn contacts(&self) -> &[DayliteContactCacheEntry] {
        &self.store.daylite_cache.contacts
    }

    async fn project_names(&self, references: HashSet<String>) -> HashMap<String, String> {
        resolve_project_names(self.app.clone(), references).await
    }
}

impl AppointmentStore for DayliteAppointments {
    async fn search_day(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<DayliteAppointment>, DayliteApiError> {
        DayliteAppointments::search_day(self, date).await
    }

    async fn create(&self, fields: &AppointmentFields) -> Result<String, DayliteApiError> {
        DayliteAppointments::create(self, fields).await
    }

    async fn patch(
        &self,
        reference: &str,
        fields: &AppointmentFields,
    ) -> Result<(), DayliteApiError> {
        DayliteAppointments::patch(self, reference, fields).await
    }

    async fn delete(&self, reference: &str) -> Result<(), DayliteApiError> {
        DayliteAppointments::delete(self, reference).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    const DATE: &str = "2026-11-02";
    const NORD: &str = "/v1/projects/42";
    const SUED: &str = "/v1/projects/43";
    const MAX: &str = "/v1/contacts/1";
    const ANNA: &str = "/v1/contacts/2";

    fn day() -> NaiveDate {
        NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap()
    }

    fn at(time: &str) -> NaiveDateTime {
        day().and_time(chrono::NaiveTime::parse_from_str(time, "%H:%M").unwrap())
    }

    fn assignment(project_ref: &str, summary: &str, start: &str, end: &str) -> RawVEvent {
        RawVEvent {
            uid: format!("{project_ref}-{start}"),
            summary: summary.to_string(),
            description: format!("daylite:{project_ref}"),
            dtstart: DATE.to_string(),
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            ..Default::default()
        }
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

    fn desired(workers: &[WorkerEvents]) -> Vec<AppointmentFields> {
        desired_appointments(day(), workers, &contacts(), &names())
    }

    fn subjects(workers: &[WorkerEvents]) -> Vec<String> {
        desired(workers)
            .into_iter()
            .map(|fields| fields.subject)
            .collect()
    }

    fn fields(project_ref: &str, subject: &str, start: &str, end: &str) -> AppointmentFields {
        AppointmentFields {
            project_ref: project_ref.to_string(),
            subject: subject.to_string(),
            local_start: at(start),
            local_end: at(end),
        }
    }

    fn existing(reference: &str, matching: &AppointmentFields) -> DayliteAppointment {
        DayliteAppointment {
            reference: reference.to_string(),
            subject: matching.subject.clone(),
            timezone: Some("floating".to_string()),
            local_start: Some(matching.local_start),
            local_end: Some(matching.local_end),
            appointment_type: Some("appointment".to_string()),
            status: Some("confirmed".to_string()),
            show_as: Some("free".to_string()),
            invitees: vec![],
            project_refs: vec![matching.project_ref.clone()],
        }
    }

    fn nord() -> AppointmentFields {
        fields(NORD, "Projekt Nord (Max Müller)", "08:00", "12:00")
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
            worker(
                ANNA,
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
        ];

        let projects: Vec<String> = desired(&workers)
            .into_iter()
            .map(|fields| fields.project_ref)
            .collect();

        assert_eq!(projects, [NORD, SUED]);
    }

    #[test]
    fn titles_the_project_with_the_assigned_workers() {
        let workers = [
            worker(
                MAX,
                vec![assignment(NORD, "Projekt Nord", "08:00", "12:00")],
            ),
            worker(
                ANNA,
                vec![assignment(NORD, "Projekt Nord", "12:00", "16:00")],
            ),
        ];

        assert_eq!(
            subjects(&workers),
            ["Projekt Nord (Anna Schmidt, Max Müller)"]
        );
    }

    #[test]
    fn lists_a_worker_with_two_assignments_once() {
        let workers = [worker(
            MAX,
            vec![
                assignment(NORD, "Projekt Nord", "08:00", "12:00"),
                assignment(NORD, "Projekt Nord", "12:00", "16:00"),
            ],
        )];

        assert_eq!(subjects(&workers), ["Projekt Nord (Max Müller)"]);
    }

    #[test]
    fn sorts_the_names_ignoring_case() {
        let contacts = [
            contact(MAX, Some("bernd Becker"), None),
            contact(ANNA, Some("Anna Schmidt"), None),
            contact("/v1/contacts/3", Some("Carla Conrad"), None),
        ];
        let workers = [
            worker(
                MAX,
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
            worker(
                ANNA,
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
            worker(
                "/v1/contacts/3",
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
        ];

        let subjects: Vec<String> = desired_appointments(day(), &workers, &contacts, &names())
            .into_iter()
            .map(|fields| fields.subject)
            .collect();

        assert_eq!(
            subjects,
            ["Projekt Nord (Anna Schmidt, bernd Becker, Carla Conrad)"]
        );
    }

    #[test]
    fn names_a_worker_by_nickname_then_full_name_then_placeholder() {
        let contacts = [
            contact(MAX, Some("Max Müller"), Some("Maxi")),
            contact(ANNA, Some("Anna Schmidt"), None),
            contact("/v1/contacts/3", None, None),
        ];
        let workers = [
            worker(
                MAX,
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
            worker(
                ANNA,
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
            worker(
                "/v1/contacts/3",
                vec![assignment(NORD, "Projekt Nord", "08:00", "16:00")],
            ),
        ];

        let subjects: Vec<String> = desired_appointments(day(), &workers, &contacts, &names())
            .into_iter()
            .map(|fields| fields.subject)
            .collect();

        assert_eq!(
            subjects,
            ["Projekt Nord (Anna Schmidt, Maxi, Unbenannter Kontakt)"]
        );
    }

    #[test]
    fn falls_back_to_the_event_summary_for_an_unresolved_project() {
        let workers = [worker(
            MAX,
            vec![assignment(
                "/v1/projects/99",
                "Projekt Ost",
                "08:00",
                "16:00",
            )],
        )];

        assert_eq!(subjects(&workers), ["Projekt Ost (Max Müller)"]);
    }

    #[test]
    fn spans_the_earliest_start_and_latest_end_across_workers() {
        let workers = [
            worker(
                MAX,
                vec![assignment(NORD, "Projekt Nord", "10:00", "12:00")],
            ),
            worker(
                ANNA,
                vec![assignment(NORD, "Projekt Nord", "08:00", "09:00")],
            ),
        ];

        let appointment = &desired(&workers)[0];

        assert_eq!(appointment.local_start, at("08:00"));
        assert_eq!(appointment.local_end, at("12:00"));
    }

    #[test]
    fn an_event_without_times_counts_as_the_full_window() {
        let workers = [worker(
            MAX,
            vec![RawVEvent {
                start_time: None,
                end_time: None,
                ..assignment(NORD, "Projekt Nord", "", "")
            }],
        )];

        let appointment = &desired(&workers)[0];

        assert_eq!(appointment.local_start, at("08:00"));
        assert_eq!(appointment.local_end, at("16:00"));
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
                    start_time: None,
                    end_time: None,
                    ..assignment(NORD, "Urlaub", "", "")
                },
                RawVEvent {
                    dtstart: "2026-11-03".to_string(),
                    ..assignment(NORD, "Projekt Nord", "08:00", "16:00")
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

        assert_eq!(
            actions,
            [AppointmentAction::Delete {
                reference: "/v1/appointments/5000".to_string()
            }]
        );
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
                    local_end: Some(at("16:00")),
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
                    invitees: vec!["/v1/users/1000".to_string()],
                    ..matching.clone()
                },
                "invitees",
            ),
            (
                DayliteAppointment {
                    project_refs: vec![NORD.to_string(), SUED.to_string()],
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
                AppointmentAction::Delete {
                    reference: "/v1/appointments/1000".to_string()
                },
                AppointmentAction::Delete {
                    reference: "/v1/appointments/1100".to_string()
                },
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
                    project_refs: vec![],
                    ..existing("/v1/appointments/5001", &nord())
                },
            ],
        );

        assert_eq!(
            actions,
            [AppointmentAction::Delete {
                reference: "/v1/appointments/5001".to_string()
            }]
        );
    }

    #[derive(Default)]
    struct StubSources {
        calendar_error: Option<String>,
        workers: Vec<(String, Vec<RawVEvent>)>,
        contacts: Vec<DayliteContactCacheEntry>,
    }

    impl DaySources for StubSources {
        async fn worker_events(&self, _date: NaiveDate) -> Result<Vec<WorkerEvents>, String> {
            if let Some(error) = &self.calendar_error {
                return Err(error.clone());
            }
            Ok(self
                .workers
                .iter()
                .map(|(reference, events)| worker(reference, events.clone()))
                .collect())
        }

        fn contacts(&self) -> &[DayliteContactCacheEntry] {
            &self.contacts
        }

        async fn project_names(&self, references: HashSet<String>) -> HashMap<String, String> {
            names()
                .into_iter()
                .filter(|(reference, _)| references.contains(reference))
                .collect()
        }
    }

    #[derive(Default)]
    struct StubAppointments {
        existing: Vec<DayliteAppointment>,
        search_fails: bool,
        failing_projects: Vec<String>,
        failing_references: Vec<String>,
        requests: std::sync::Mutex<Vec<String>>,
    }

    impl StubAppointments {
        fn log(&self, request: String) {
            self.requests.lock().unwrap().push(request);
        }

        fn requests(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }
    }

    fn unreachable_daylite() -> DayliteApiError {
        DayliteApiError {
            code: crate::integrations::daylite::shared::DayliteApiErrorCode::ServerError,
            http_status: Some(503),
            user_message: "Daylite ist aktuell nicht erreichbar.".to_string(),
            technical_message: "status=503".to_string(),
        }
    }

    impl AppointmentStore for StubAppointments {
        async fn search_day(
            &self,
            date: NaiveDate,
        ) -> Result<Vec<DayliteAppointment>, DayliteApiError> {
            self.log(format!("search {date}"));
            if self.search_fails {
                return Err(unreachable_daylite());
            }
            Ok(self.existing.clone())
        }

        async fn create(&self, fields: &AppointmentFields) -> Result<String, DayliteApiError> {
            self.log(format!("create {}", fields.project_ref));
            if self.failing_projects.contains(&fields.project_ref) {
                return Err(unreachable_daylite());
            }
            Ok("/v1/appointments/7000".to_string())
        }

        async fn patch(
            &self,
            reference: &str,
            _fields: &AppointmentFields,
        ) -> Result<(), DayliteApiError> {
            self.log(format!("patch {reference}"));
            if self
                .failing_references
                .iter()
                .any(|failing| failing == reference)
            {
                return Err(unreachable_daylite());
            }
            Ok(())
        }

        async fn delete(&self, reference: &str) -> Result<(), DayliteApiError> {
            self.log(format!("delete {reference}"));
            if self
                .failing_references
                .iter()
                .any(|failing| failing == reference)
            {
                return Err(unreachable_daylite());
            }
            Ok(())
        }
    }

    fn two_projects() -> StubSources {
        StubSources {
            workers: vec![
                (
                    MAX.to_string(),
                    vec![assignment(NORD, "Projekt Nord", "08:00", "12:00")],
                ),
                (
                    ANNA.to_string(),
                    vec![assignment(SUED, "Projekt Süd", "08:00", "16:00")],
                ),
            ],
            contacts: contacts(),
            ..Default::default()
        }
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

    fn sync_errors(
        receiver: &mut tokio::sync::mpsc::Receiver<
            crate::integrations::telemetry::recorder::TelemetryCommand,
        >,
    ) -> Vec<(String, String)> {
        crate::integrations::telemetry::test_support::recorded(receiver)
            .into_iter()
            .filter(|event| event.name() == "error_occurred")
            .map(|event| {
                (
                    event.properties()["operation"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    event.properties()["integration"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                )
            })
            .collect()
    }

    #[tokio::test]
    async fn an_unreadable_worker_calendar_sends_no_daylite_request() {
        let (recorder, mut receiver) =
            crate::integrations::telemetry::test_support::enabled_recorder();
        let sources = StubSources {
            calendar_error: Some("CalDAV-Server antwortete mit HTTP 500".to_string()),
            ..two_projects()
        };
        let appointments = StubAppointments::default();

        let synced = reconcile_day(day(), &sources, &appointments, Some(&recorder)).await;

        assert!(!synced);
        assert!(appointments.requests().is_empty());
        assert_eq!(
            sync_errors(&mut receiver),
            [("daylite_appointment_sync".to_string(), "zep".to_string())]
        );
    }

    #[tokio::test]
    async fn a_failed_create_still_applies_the_other_projects_actions() {
        let (recorder, mut receiver) =
            crate::integrations::telemetry::test_support::enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![stale("/v1/appointments/5000")],
            failing_projects: vec![NORD.to_string()],
            ..Default::default()
        };

        let synced = reconcile_day(day(), &two_projects(), &appointments, Some(&recorder)).await;

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
            [(
                "daylite_appointment_sync".to_string(),
                "daylite".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn every_daylite_failure_is_recorded_under_the_sync_operation() {
        let (recorder, mut receiver) =
            crate::integrations::telemetry::test_support::enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![
                DayliteAppointment {
                    status: Some("tentative".to_string()),
                    ..existing(
                        "/v1/appointments/5000",
                        &fields(NORD, "Projekt Nord (Max Müller)", "08:00", "12:00"),
                    )
                },
                stale("/v1/appointments/5001"),
            ],
            failing_references: vec![
                "/v1/appointments/5000".to_string(),
                "/v1/appointments/5001".to_string(),
            ],
            ..Default::default()
        };

        let synced = reconcile_day(day(), &two_projects(), &appointments, Some(&recorder)).await;

        assert!(!synced);
        let daylite_failure = (
            "daylite_appointment_sync".to_string(),
            "daylite".to_string(),
        );
        assert_eq!(
            sync_errors(&mut receiver),
            [daylite_failure.clone(), daylite_failure]
        );
    }

    #[tokio::test]
    async fn a_failed_search_changes_nothing() {
        let (recorder, mut receiver) =
            crate::integrations::telemetry::test_support::enabled_recorder();
        let appointments = StubAppointments {
            search_fails: true,
            ..Default::default()
        };

        let synced = reconcile_day(day(), &two_projects(), &appointments, Some(&recorder)).await;

        assert!(!synced);
        assert_eq!(appointments.requests(), ["search 2026-11-02"]);
        assert_eq!(
            sync_errors(&mut receiver),
            [(
                "daylite_appointment_sync".to_string(),
                "daylite".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn a_day_that_converges_reports_success_without_errors() {
        let (recorder, mut receiver) =
            crate::integrations::telemetry::test_support::enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![existing(
                "/v1/appointments/5000",
                &fields(NORD, "Projekt Nord (Max Müller)", "08:00", "12:00"),
            )],
            ..Default::default()
        };

        let synced = reconcile_day(day(), &two_projects(), &appointments, Some(&recorder)).await;

        assert!(synced);
        assert_eq!(
            appointments.requests(),
            ["search 2026-11-02", "create /v1/projects/43"]
        );
        assert!(sync_errors(&mut receiver).is_empty());
    }

    #[tokio::test]
    async fn reconciliations_of_one_date_run_one_after_the_other() {
        let sync = AppointmentSync::default();
        let log = Mutex::new(Vec::new());
        let (release, released) = tokio::sync::oneshot::channel::<()>();
        let other_day = NaiveDate::from_ymd_opt(2026, 11, 3).unwrap();

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
                other_day,
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

    fn ignore(_: Vec<String>) {}

    fn next_day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 3).unwrap()
    }

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

    #[tokio::test]
    async fn a_failed_reconciliation_lists_its_date() {
        let sync = AppointmentSync::default();
        let notifications = Notifications::default();

        settle(&sync, next_day(), false, &notifications.notify()).await;
        settle(&sync, day(), false, &notifications.notify()).await;

        assert_eq!(
            notifications.received(),
            [
                vec!["2026-11-03".to_string()],
                vec!["2026-11-02".to_string(), "2026-11-03".to_string()],
            ]
        );
    }

    #[tokio::test]
    async fn a_successful_reconciliation_clears_its_date() {
        let sync = AppointmentSync::default();
        let notifications = Notifications::default();

        settle(&sync, day(), false, &notifications.notify()).await;
        settle(&sync, day(), true, &notifications.notify()).await;

        assert_eq!(
            notifications.received(),
            [vec!["2026-11-02".to_string()], vec![]]
        );
    }

    #[tokio::test]
    async fn an_unchanged_set_of_failed_dates_is_not_announced_again() {
        let sync = AppointmentSync::default();
        let notifications = Notifications::default();

        settle(&sync, day(), true, &notifications.notify()).await;
        settle(&sync, day(), false, &notifications.notify()).await;
        settle(&sync, day(), false, &notifications.notify()).await;

        assert_eq!(notifications.received(), [vec!["2026-11-02".to_string()]]);
    }

    #[tokio::test]
    async fn a_day_with_an_unreadable_worker_calendar_counts_as_failed() {
        let sync = AppointmentSync::default();
        let notifications = Notifications::default();
        let sources = StubSources {
            calendar_error: Some("CalDAV-Server antwortete mit HTTP 500".to_string()),
            ..two_projects()
        };

        let appointments = StubAppointments::default();

        sync.run(
            day(),
            || reconcile_day(day(), &sources, &appointments, None),
            &notifications.notify(),
        )
        .await;

        assert_eq!(notifications.received(), [vec!["2026-11-02".to_string()]]);
    }

    async fn failing(dates: &[NaiveDate]) -> AppointmentSync {
        let sync = AppointmentSync::default();
        for date in dates {
            settle(&sync, *date, false, &ignore).await;
        }
        sync
    }

    #[tokio::test]
    async fn a_retry_reconciles_every_failed_date() {
        let sync = failing(&[day(), next_day()]).await;
        let notifications = Notifications::default();
        let retried = Mutex::new(Vec::new());

        sync.retry_failed(
            |date| {
                retried.lock().unwrap().push(date);
                async { true }
            },
            &notifications.notify(),
        )
        .await;

        let mut retried = retried.lock().unwrap().clone();
        retried.sort();
        assert_eq!(retried, [day(), next_day()]);
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
    async fn a_retry_resolves_once_every_date_finished() {
        let sync = failing(&[day(), next_day()]).await;
        let finished = std::sync::atomic::AtomicUsize::new(0);

        sync.retry_failed(
            |_| async {
                tokio::task::yield_now().await;
                finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                true
            },
            &ignore,
        )
        .await;

        assert_eq!(finished.load(std::sync::atomic::Ordering::SeqCst), 2);
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
}
