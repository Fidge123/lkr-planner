use chrono::NaiveDate;
use futures::StreamExt;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};

use super::caldav::{fetch_events_in_range, lies_in_calendar, resolve_href, CaldavSession};
use super::commands::{load_caldav_session, resolve_employee_calendar_url};
use super::events::classify_event;
use super::slots::full_window;
use super::types::RawVEvent;
use crate::integrations::daylite::appointments::{
    AppointmentFields, AppointmentStore, DayliteAppointment, DayliteAppointments,
};
use crate::integrations::daylite::shared::DayliteApiError;
use crate::integrations::local_store::LocalStore;
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::{
    record_failure, record_failure_with, RequestFailure, TelemetryError,
};
use crate::integrations::telemetry::recorder::TelemetryRecorder;
use crate::integrations::telemetry::state::TelemetryState;

/// A worker whose day a write touched; a synchronization only changes this worker's links.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct WorkerDay {
    pub(super) worker: String,
    pub(super) date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AppointmentAction {
    /// Creates the project's appointment linked to the worker.
    Create(AppointmentFields),
    /// Restores the owned fields and links the given contacts, then deletes the duplicates it replaces.
    Update {
        reference: String,
        fields: AppointmentFields,
        link: Vec<String>,
        replaces: Vec<String>,
    },
    /// Unlinks the worker.
    Unlink {
        reference: String,
    },
    Delete {
        reference: String,
    },
}

pub(super) fn appointment_fields(project_ref: &str, date: NaiveDate) -> AppointmentFields {
    let (start, end) = full_window();
    AppointmentFields {
        project_ref: project_ref.to_string(),
        subject: format!("Einsatz {}", date.format("%d.%m.%Y")),
        local_start: date.and_time(start),
        local_end: date.and_time(end),
    }
}

fn worker_projects(date: NaiveDate, events: &[RawVEvent]) -> BTreeSet<String> {
    let iso_date = date.format("%Y-%m-%d").to_string();
    events
        .iter()
        .map(classify_event)
        .filter(|event| event.date == iso_date)
        .filter_map(|event| event.project_ref)
        .collect()
}

/// Only the appointments of the worker's projects and the appointments linked to the worker are touched.
/// Of several appointments for one project the lowest ID is kept and takes over the others' contacts.
/// An appointment whose last planning contact is the worker is deleted rather than unlinked.
pub(super) fn plan_worker_actions(
    worker_day: &WorkerDay,
    projects: &BTreeSet<String>,
    existing: Vec<DayliteAppointment>,
    planning_contacts: &HashSet<String>,
) -> Vec<AppointmentAction> {
    let worker = worker_day.worker.as_str();
    let mut by_project: BTreeMap<Option<String>, Vec<DayliteAppointment>> = BTreeMap::new();
    for appointment in existing {
        let project_ref = appointment.project_refs().next().map(str::to_string);
        by_project.entry(project_ref).or_default().push(appointment);
    }

    let mut actions = Vec::new();
    for project_ref in projects {
        let fields = appointment_fields(project_ref, worker_day.date);
        let mut candidates = by_project
            .remove(&Some(project_ref.clone()))
            .unwrap_or_default();
        candidates.sort_by_key(DayliteAppointment::numeric_id);
        let mut candidates = candidates.into_iter();
        let Some(kept) = candidates.next() else {
            actions.push(AppointmentAction::Create(fields));
            continue;
        };

        let duplicates: Vec<DayliteAppointment> = candidates.collect();
        let linked: HashSet<&str> = kept.contact_refs().collect();
        let link: Vec<String> = std::iter::once(worker)
            .chain(duplicates.iter().flat_map(DayliteAppointment::contact_refs))
            .filter(|contact| !linked.contains(contact))
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if link.is_empty() && kept.matches(&fields) {
            actions.extend(duplicates.into_iter().map(delete));
        } else {
            actions.push(AppointmentAction::Update {
                reference: kept.reference.clone(),
                fields,
                link,
                replaces: duplicates
                    .into_iter()
                    .map(|duplicate| duplicate.reference)
                    .collect(),
            });
        }
    }

    for appointment in by_project.into_values().flatten() {
        if !appointment.contact_refs().any(|contact| contact == worker) {
            continue;
        }
        let others_planned = appointment
            .contact_refs()
            .any(|contact| contact != worker && planning_contacts.contains(contact));
        actions.push(if others_planned {
            AppointmentAction::Unlink {
                reference: appointment.reference,
            }
        } else {
            delete(appointment)
        });
    }

    actions
}

fn delete(appointment: DayliteAppointment) -> AppointmentAction {
    AppointmentAction::Delete {
        reference: appointment.reference,
    }
}

/// Nothing is written unless the worker's calendar was read, because a missing view would unlink the worker everywhere.
/// One failed action does not stop the others, but fails the worker's day.
pub(super) async fn reconcile_worker_day(
    worker_day: &WorkerDay,
    events: Result<Vec<RawVEvent>, String>,
    planning_contacts: &HashSet<String>,
    appointments: &impl AppointmentStore,
    telemetry: Option<&TelemetryRecorder>,
) -> bool {
    let WorkerDay { worker, date } = worker_day;
    let events = match events {
        Ok(events) => events,
        Err(error) => {
            let failure =
                RequestFailure::new("CALENDAR_READ_FAILED", format!("{worker} {date}: {error}"));
            return sync_failed(telemetry, Integration::Zep, &failure);
        }
    };

    let existing = match appointments.search_day(*date).await {
        Ok(existing) => existing,
        Err(error) => return sync_failed(telemetry, Integration::Daylite, &error),
    };

    let projects = worker_projects(*date, &events);
    let mut synced = true;
    let mut succeeded = |result: Result<(), DayliteApiError>| match result {
        Ok(()) => true,
        Err(error) => {
            synced = sync_failed(telemetry, Integration::Daylite, &error);
            false
        }
    };
    for action in plan_worker_actions(worker_day, &projects, existing, planning_contacts) {
        match action {
            AppointmentAction::Create(fields) => {
                succeeded(appointments.create(&fields, worker).await.map(drop));
            }
            AppointmentAction::Update {
                reference,
                fields,
                link,
                replaces,
            } => {
                if succeeded(appointments.update(&reference, &fields, &link).await) {
                    for duplicate in replaces {
                        succeeded(appointments.delete(&duplicate).await);
                    }
                }
            }
            AppointmentAction::Unlink { reference } => {
                succeeded(appointments.unlink(&reference, worker).await);
            }
            AppointmentAction::Delete { reference } => {
                succeeded(appointments.delete(&reference).await);
            }
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

/// Serializes synchronizations per date, so two quick writes cannot both find no appointment and create one each.
#[derive(Default)]
pub(super) struct AppointmentSync {
    locks: Mutex<HashMap<NaiveDate, Arc<tokio::sync::Mutex<()>>>>,
    failed: Mutex<BTreeSet<WorkerDay>>,
}

impl AppointmentSync {
    /// `notify` receives the sorted ISO dates still failing whenever the failed worker days change.
    pub(super) async fn run<Fut, N>(
        &self,
        worker_day: WorkerDay,
        reconcile: impl FnOnce() -> Fut,
        notify: &N,
    ) -> bool
    where
        Fut: Future<Output = bool>,
        N: Fn(Vec<String>),
    {
        let lock = self.lock_for(worker_day.date);
        let _guard = lock.lock().await;
        let synced = reconcile().await;
        self.record_outcome(worker_day, synced, notify);
        synced
    }

    /// Announces under the lock, so two outcomes finishing together cannot reach the frontend out of order.
    fn record_outcome(&self, worker_day: WorkerDay, synced: bool, notify: &impl Fn(Vec<String>)) {
        let mut failed = self
            .failed
            .lock()
            .expect("appointment sync failures poisoned");
        let changed = if synced {
            failed.remove(&worker_day)
        } else {
            failed.insert(worker_day)
        };
        if changed {
            let dates: BTreeSet<NaiveDate> = failed.iter().map(|failed| failed.date).collect();
            notify(dates.iter().map(NaiveDate::to_string).collect());
        }
    }

    /// Synchronizes every worker day failing when called, each under its date's lock.
    pub(super) async fn retry_failed<Fut, N>(
        &self,
        reconcile: impl Fn(WorkerDay) -> Fut,
        notify: &N,
    ) where
        Fut: Future<Output = bool>,
        N: Fn(Vec<String>),
    {
        let failed: Vec<WorkerDay> = self
            .failed
            .lock()
            .expect("appointment sync failures poisoned")
            .iter()
            .cloned()
            .collect();

        let reconcile = &reconcile;
        futures::stream::iter(failed)
            .map(|worker_day| {
                let retried = worker_day.clone();
                self.run(worker_day, move || reconcile(retried), notify)
            })
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
pub(super) fn schedule_appointment_sync(app: &tauri::AppHandle, worker_days: Vec<WorkerDay>) {
    for worker_day in worker_days {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let synchronized = worker_day.clone();
            appointment_sync()
                .run(
                    worker_day,
                    || reconcile_in_app(&app, synchronized),
                    &failure_notifier(&app),
                )
                .await;
        });
    }
}

pub(super) async fn retry_failed_in_app(app: &tauri::AppHandle) {
    appointment_sync()
        .retry_failed(
            |worker_day| reconcile_in_app(app, worker_day),
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

/// Reads the configuration afresh, so a settings change applies to the next synchronization.
async fn reconcile_in_app(app: &tauri::AppHandle, worker_day: WorkerDay) -> bool {
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

    let next_day = worker_day.date + chrono::Duration::days(1);
    let events = async {
        let calendar_url = resolve_employee_calendar_url(&store, &worker_day.worker)?;
        fetch_events_in_range(&session, &calendar_url, worker_day.date, next_day).await
    }
    .await;
    let planning_contacts: HashSet<String> = store
        .daylite_cache
        .contacts
        .iter()
        .map(|contact| contact.reference.clone())
        .collect();

    reconcile_worker_day(
        &worker_day,
        events,
        &planning_contacts,
        &appointments,
        telemetry,
    )
    .await
}

/// Records a failure when no configured primary calendar holds the event, since its worker's links then stay unsynchronized.
pub(super) fn event_worker(
    app: &tauri::AppHandle,
    store: &LocalStore,
    session: &CaldavSession,
    href: &str,
) -> Option<String> {
    let worker = worker_of_href(store, &session.base_url, href);
    if worker.is_none() {
        record_failure(
            app,
            Operation::DayliteAppointmentSync,
            Integration::Zep,
            &RequestFailure::new("WORKER_NOT_FOUND", href),
        );
    }
    worker
}

fn worker_of_href(store: &LocalStore, base_url: &str, href: &str) -> Option<String> {
    let resource_url = resolve_href(href, base_url).ok()?;
    store
        .employee_settings
        .iter()
        .find(|setting| {
            setting
                .zep_primary_calendar
                .as_deref()
                .is_some_and(|calendar| lies_in_calendar(&resource_url, calendar))
        })
        .map(|setting| setting.daylite_contact_reference.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::daylite::appointments::{DayliteContactLink, DayliteProjectLink};
    use crate::integrations::daylite::shared::DayliteApiErrorCode;
    use crate::integrations::local_store::EmployeeSetting;
    use crate::integrations::telemetry::recorder::TelemetryCommand;
    use crate::integrations::telemetry::test_support::{enabled_recorder, recorded};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    const DATE: &str = "2026-11-02";
    const NORD: &str = "/v1/projects/42";
    const SUED: &str = "/v1/projects/43";
    const WEST: &str = "/v1/projects/44";
    const MAX: &str = "/v1/contacts/1";
    const ANNA: &str = "/v1/contacts/2";
    const THIRD: &str = "/v1/contacts/3";
    const OUTSIDER: &str = "/v1/contacts/900";

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()
    }

    fn next_day() -> NaiveDate {
        day() + chrono::Duration::days(1)
    }

    fn on(worker: &str, date: NaiveDate) -> WorkerDay {
        WorkerDay {
            worker: worker.to_string(),
            date,
        }
    }

    fn max_today() -> WorkerDay {
        on(MAX, day())
    }

    fn assignment(project_ref: &str, start: &str) -> RawVEvent {
        RawVEvent {
            uid: format!("{project_ref}-{start}"),
            summary: "Projekt".to_string(),
            description: format!("daylite:{project_ref}"),
            dtstart: DATE.to_string(),
            start_time: Some(start.to_string()),
            ..Default::default()
        }
    }

    fn planning() -> HashSet<String> {
        [MAX, ANNA, THIRD].map(str::to_string).into()
    }

    fn projects(references: &[&str]) -> BTreeSet<String> {
        references
            .iter()
            .map(|reference| reference.to_string())
            .collect()
    }

    fn fields(project_ref: &str) -> AppointmentFields {
        appointment_fields(project_ref, day())
    }

    fn existing(reference: &str, project_ref: &str, contacts: &[&str]) -> DayliteAppointment {
        let matching = fields(project_ref);
        let local =
            |time: chrono::NaiveDateTime| Some(time.format("%Y-%m-%dT%H:%M:%S").to_string());
        DayliteAppointment {
            reference: reference.to_string(),
            subject: matching.subject,
            timezone: Some("floating".to_string()),
            local_start: local(matching.local_start),
            local_end: local(matching.local_end),
            appointment_type: Some("appointment".to_string()),
            status: Some("confirmed".to_string()),
            show_as: Some("free".to_string()),
            invitations: Some(vec![]),
            projects: Some(vec![DayliteProjectLink {
                project: project_ref.to_string(),
            }]),
            contacts: Some(
                contacts
                    .iter()
                    .map(|contact| DayliteContactLink {
                        contact: contact.to_string(),
                    })
                    .collect(),
            ),
        }
    }

    fn plan(worker_projects: &[&str], existing: Vec<DayliteAppointment>) -> Vec<AppointmentAction> {
        plan_worker_actions(
            &max_today(),
            &projects(worker_projects),
            existing,
            &planning(),
        )
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn update(reference: &str, project_ref: &str, link: &[&str]) -> AppointmentAction {
        AppointmentAction::Update {
            reference: reference.to_string(),
            fields: fields(project_ref),
            link: strings(link),
            replaces: vec![],
        }
    }

    fn deleted(reference: &str) -> AppointmentAction {
        AppointmentAction::Delete {
            reference: reference.to_string(),
        }
    }

    #[test]
    fn appointments_are_titled_after_their_day_and_run_from_eight_to_four() {
        let fields = fields(NORD);

        assert_eq!(fields.subject, "Einsatz 02.11.2026");
        assert_eq!(fields.local_start, day().and_hms_opt(8, 0, 0).unwrap());
        assert_eq!(fields.local_end, day().and_hms_opt(16, 0, 0).unwrap());
    }

    #[test]
    fn the_workers_projects_come_from_its_assignments_of_the_day() {
        let events = [
            assignment(NORD, "08:00"),
            assignment(NORD, "12:00"),
            RawVEvent {
                description: "Bitte Auto abholen".to_string(),
                ..assignment(SUED, "09:00")
            },
            RawVEvent {
                dtstart: "2026-11-03".to_string(),
                ..assignment(WEST, "08:00")
            },
        ];

        assert_eq!(worker_projects(day(), &events), projects(&[NORD]));
    }

    #[test]
    fn creates_the_appointment_of_a_project_without_one() {
        assert_eq!(
            plan(&[NORD], vec![]),
            [AppointmentAction::Create(fields(NORD))]
        );
    }

    #[test]
    fn links_the_worker_to_the_existing_appointment_of_its_project() {
        assert_eq!(
            plan(
                &[NORD],
                vec![existing("/v1/appointments/5000", NORD, &[ANNA])]
            ),
            [update("/v1/appointments/5000", NORD, &[MAX])]
        );
    }

    #[test]
    fn leaves_an_appointment_alone_that_matches_and_links_the_worker() {
        assert!(plan(
            &[NORD],
            vec![existing("/v1/appointments/5000", NORD, &[MAX])]
        )
        .is_empty());
    }

    #[test]
    fn restores_an_owned_field_that_differs() {
        let matching = existing("/v1/appointments/5000", NORD, &[MAX]);
        let project = |project: &str| DayliteProjectLink {
            project: project.to_string(),
        };
        let cases = [
            (
                DayliteAppointment {
                    subject: "Projekt Nord".to_string(),
                    ..matching.clone()
                },
                "subject",
            ),
            (
                DayliteAppointment {
                    local_end: Some("2026-11-02T12:00:00".to_string()),
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
                    invitations: Some(vec![serde_json::json!({ "invitee": MAX })]),
                    ..matching.clone()
                },
                "invitees",
            ),
            (
                DayliteAppointment {
                    projects: Some(vec![project(NORD), project(SUED)]),
                    ..matching.clone()
                },
                "projects",
            ),
        ];

        for (appointment, label) in cases {
            assert_eq!(
                plan(&[NORD], vec![appointment]),
                [update("/v1/appointments/5000", NORD, &[])],
                "case: {label}"
            );
        }
    }

    #[test]
    fn a_worker_that_left_a_project_is_unlinked_or_its_appointment_deleted() {
        let without_project = DayliteAppointment {
            projects: None,
            ..existing("/v1/appointments/5002", WEST, &[MAX])
        };
        let cases = [
            (
                "another planning contact stays linked",
                existing("/v1/appointments/5000", NORD, &[MAX, ANNA]),
                AppointmentAction::Unlink {
                    reference: "/v1/appointments/5000".to_string(),
                },
            ),
            (
                "only a contact outside the planning list stays linked",
                existing("/v1/appointments/5001", NORD, &[MAX, OUTSIDER]),
                deleted("/v1/appointments/5001"),
            ),
            (
                "an appointment without a project",
                without_project,
                deleted("/v1/appointments/5002"),
            ),
        ];

        for (label, appointment, expected) in cases {
            assert_eq!(plan(&[], vec![appointment]), [expected], "case: {label}");
        }
    }

    #[test]
    fn leaves_the_appointments_of_other_workers_alone() {
        assert!(plan(
            &[NORD],
            vec![
                existing("/v1/appointments/5000", NORD, &[MAX]),
                existing("/v1/appointments/5001", SUED, &[ANNA]),
            ]
        )
        .is_empty());
    }

    #[test]
    fn keeps_the_lowest_id_of_duplicates_and_takes_over_their_contacts() {
        let actions = plan(
            &[NORD],
            vec![
                existing("/v1/appointments/1000", NORD, &[ANNA]),
                existing("/v1/appointments/900", NORD, &[MAX]),
                existing("/v1/appointments/1100", NORD, &[THIRD, MAX]),
            ],
        );

        assert_eq!(
            actions,
            [AppointmentAction::Update {
                reference: "/v1/appointments/900".to_string(),
                fields: fields(NORD),
                link: strings(&[ANNA, THIRD]),
                replaces: strings(&["/v1/appointments/1000", "/v1/appointments/1100"]),
            }]
        );
    }

    #[test]
    fn deletes_duplicates_whose_contacts_the_kept_appointment_already_links() {
        let actions = plan(
            &[NORD],
            vec![
                existing("/v1/appointments/900", NORD, &[MAX, ANNA]),
                existing("/v1/appointments/1000", NORD, &[ANNA]),
            ],
        );

        assert_eq!(actions, [deleted("/v1/appointments/1000")]);
    }

    #[derive(Default)]
    struct StubAppointments {
        existing: Vec<DayliteAppointment>,
        search_fails: bool,
        /// Project references whose create fails, and appointment references whose other requests fail.
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

        async fn create(
            &self,
            fields: &AppointmentFields,
            contact: &str,
        ) -> Result<String, DayliteApiError> {
            self.respond(
                format!("create {} {contact}", fields.project_ref),
                self.fails(&fields.project_ref),
                "/v1/appointments/7000".to_string(),
            )
        }

        async fn update(
            &self,
            reference: &str,
            _: &AppointmentFields,
            link: &[String],
        ) -> Result<(), DayliteApiError> {
            self.respond(
                format!("update {reference} {}", link.join(" ")),
                self.fails(reference),
                (),
            )
        }

        async fn unlink(&self, reference: &str, contact: &str) -> Result<(), DayliteApiError> {
            self.respond(
                format!("unlink {reference} {contact}"),
                self.fails(reference),
                (),
            )
        }

        async fn delete(&self, reference: &str) -> Result<(), DayliteApiError> {
            self.respond(format!("delete {reference}"), self.fails(reference), ())
        }
    }

    fn unreadable_calendar() -> Result<Vec<RawVEvent>, String> {
        Err("CalDAV-Server antwortete mit HTTP 500".to_string())
    }

    async fn reconcile(
        events: Result<Vec<RawVEvent>, String>,
        appointments: &StubAppointments,
        telemetry: Option<&TelemetryRecorder>,
    ) -> bool {
        reconcile_worker_day(&max_today(), events, &planning(), appointments, telemetry).await
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
    async fn a_failed_create_still_applies_the_other_actions() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![existing("/v1/appointments/5000", WEST, &[MAX])],
            failing: vec![NORD.to_string()],
            ..Default::default()
        };

        let synced = reconcile(
            Ok(vec![assignment(NORD, "08:00"), assignment(SUED, "12:00")]),
            &appointments,
            Some(&recorder),
        )
        .await;

        assert!(!synced);
        assert_eq!(
            appointments.requests(),
            [
                "search 2026-11-02",
                "create /v1/projects/42 /v1/contacts/1",
                "create /v1/projects/43 /v1/contacts/1",
                "delete /v1/appointments/5000",
            ]
        );
        assert_eq!(
            sync_errors(&mut receiver),
            ["daylite_appointment_sync daylite"]
        );
    }

    #[tokio::test]
    async fn duplicates_survive_a_failed_takeover_of_their_contacts() {
        let appointments = StubAppointments {
            existing: vec![
                existing("/v1/appointments/900", NORD, &[MAX]),
                existing("/v1/appointments/1000", NORD, &[ANNA]),
            ],
            failing: vec!["/v1/appointments/900".to_string()],
            ..Default::default()
        };

        let synced = reconcile(Ok(vec![assignment(NORD, "08:00")]), &appointments, None).await;

        assert!(!synced);
        assert_eq!(
            appointments.requests(),
            [
                "search 2026-11-02",
                "update /v1/appointments/900 /v1/contacts/2"
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

        let synced = reconcile(
            Ok(vec![assignment(NORD, "08:00")]),
            &appointments,
            Some(&recorder),
        )
        .await;

        assert!(!synced);
        assert_eq!(appointments.requests(), ["search 2026-11-02"]);
        assert_eq!(
            sync_errors(&mut receiver),
            ["daylite_appointment_sync daylite"]
        );
    }

    #[tokio::test]
    async fn a_worker_day_that_converges_reports_success_without_errors() {
        let (recorder, mut receiver) = enabled_recorder();
        let appointments = StubAppointments {
            existing: vec![
                existing("/v1/appointments/5000", NORD, &[MAX]),
                existing("/v1/appointments/5001", SUED, &[ANNA]),
            ],
            ..Default::default()
        };

        let synced = reconcile(
            Ok(vec![assignment(NORD, "08:00"), assignment(WEST, "12:00")]),
            &appointments,
            Some(&recorder),
        )
        .await;

        assert!(synced);
        assert_eq!(
            appointments.requests(),
            ["search 2026-11-02", "create /v1/projects/44 /v1/contacts/1"]
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
        worker_day: WorkerDay,
        synced: bool,
        notify: &impl Fn(Vec<String>),
    ) {
        sync.run(worker_day, || async move { synced }, notify).await;
    }

    async fn failing(worker_days: &[WorkerDay]) -> AppointmentSync {
        let sync = AppointmentSync::default();
        for worker_day in worker_days {
            settle(&sync, worker_day.clone(), false, &ignore).await;
        }
        sync
    }

    #[tokio::test]
    async fn synchronizations_of_one_date_run_one_after_the_other() {
        let sync = AppointmentSync::default();
        let log = Mutex::new(Vec::new());
        let (release, released) = tokio::sync::oneshot::channel::<()>();

        tokio::join!(
            sync.run(
                on(MAX, day()),
                || async {
                    log.lock().unwrap().push("first started");
                    released.await.unwrap();
                    log.lock().unwrap().push("first finished");
                    true
                },
                &ignore,
            ),
            sync.run(
                on(ANNA, day()),
                || async {
                    log.lock().unwrap().push("second started");
                    true
                },
                &ignore,
            ),
            sync.run(
                on(MAX, next_day()),
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
                vec![(on(MAX, next_day()), false), (on(MAX, day()), false)],
                vec![dates(&["2026-11-03"]), dates(&["2026-11-02", "2026-11-03"])],
            ),
            (
                "a success clears its date",
                vec![(on(MAX, day()), false), (on(MAX, day()), true)],
                vec![dates(&["2026-11-02"]), vec![]],
            ),
            (
                "a date stays listed while another worker on it still fails",
                vec![
                    (on(MAX, day()), false),
                    (on(ANNA, day()), false),
                    (on(MAX, day()), true),
                ],
                vec![
                    dates(&["2026-11-02"]),
                    dates(&["2026-11-02"]),
                    dates(&["2026-11-02"]),
                ],
            ),
            (
                "an unchanged set is not announced again",
                vec![
                    (on(MAX, day()), true),
                    (on(MAX, day()), false),
                    (on(MAX, day()), false),
                ],
                vec![dates(&["2026-11-02"])],
            ),
        ];

        for (label, outcomes, expected) in cases {
            let sync = AppointmentSync::default();
            let notifications = Notifications::default();
            for (worker_day, synced) in outcomes {
                settle(&sync, worker_day, synced, &notifications.notify()).await;
            }

            assert_eq!(notifications.received(), expected, "case: {label}");
        }
    }

    #[tokio::test]
    async fn a_worker_day_with_an_unreadable_calendar_counts_as_failed() {
        let sync = AppointmentSync::default();
        let notifications = Notifications::default();
        let appointments = StubAppointments::default();

        sync.run(
            max_today(),
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

        settle(&sync, max_today(), false, &|_| {
            registry_held.store(sync.failed.try_lock().is_err(), Ordering::SeqCst)
        })
        .await;

        assert!(registry_held.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn a_retry_synchronizes_every_failed_worker_day_before_it_resolves() {
        let sync = failing(&[on(MAX, day()), on(ANNA, next_day())]).await;
        let notifications = Notifications::default();
        let finished = Mutex::new(Vec::new());

        sync.retry_failed(
            |worker_day| {
                let finished = &finished;
                async move {
                    tokio::task::yield_now().await;
                    finished.lock().unwrap().push(worker_day);
                    true
                }
            },
            &notifications.notify(),
        )
        .await;

        let mut finished = finished.lock().unwrap().clone();
        finished.sort();
        assert_eq!(finished, [on(MAX, day()), on(ANNA, next_day())]);
        assert_eq!(notifications.received().last(), Some(&vec![]));
    }

    #[tokio::test]
    async fn a_retry_keeps_only_the_worker_days_that_fail_again() {
        let sync = failing(&[on(MAX, day()), on(MAX, next_day())]).await;
        let notifications = Notifications::default();

        sync.retry_failed(
            |worker_day| async move { worker_day.date == day() },
            &notifications.notify(),
        )
        .await;

        assert_eq!(notifications.received(), [vec!["2026-11-03".to_string()]]);
    }

    #[tokio::test]
    async fn a_retry_waits_for_a_write_synchronizing_the_same_date() {
        let sync = failing(&[on(MAX, day())]).await;
        let log = Mutex::new(Vec::new());
        let (release, released) = tokio::sync::oneshot::channel::<()>();

        tokio::join!(
            sync.run(
                on(ANNA, day()),
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
    async fn a_retry_synchronizes_at_most_two_worker_days_at_once() {
        let worker_days: Vec<WorkerDay> = (0..5)
            .map(|offset| on(MAX, day() + chrono::Duration::days(offset)))
            .collect();
        let sync = failing(&worker_days).await;
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

    #[test]
    fn an_event_belongs_to_the_worker_whose_primary_calendar_holds_it() {
        let store = LocalStore {
            employee_settings: [
                (MAX, "https://zep.example/caldav/max/"),
                (ANNA, "https://zep.example/caldav/anna"),
            ]
            .map(|(reference, calendar)| EmployeeSetting {
                daylite_contact_reference: reference.to_string(),
                zep_primary_calendar: Some(calendar.to_string()),
                ..Default::default()
            })
            .into(),
            ..Default::default()
        };
        let worker = |href: &str| worker_of_href(&store, "https://zep.example/caldav", href);

        assert_eq!(worker("/caldav/max/uid-1.ics").as_deref(), Some(MAX));
        assert_eq!(worker("/caldav/anna/uid-2.ics").as_deref(), Some(ANNA));
        assert_eq!(worker("/caldav/maxi/uid-3.ics"), None);
    }
}
