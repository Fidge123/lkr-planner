use chrono::NaiveDate;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager};

use super::caldav::{fetch_events_in_range, lies_in_calendar, resolve_href, CaldavSession};
use super::commands::{load_caldav_session, resolve_employee_calendar_url};
use super::events::classify_event;
use super::slots::full_window;
use super::types::RawVEvent;
use crate::error::AppError;
use crate::integrations::daylite::appointments::{
    AppointmentFields, AppointmentStore, DayliteAppointment, DayliteAppointments,
};
use crate::integrations::local_store::LocalStore;
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::{
    record_failure, record_failure_with, RequestFailure, TelemetryError,
};
use crate::integrations::telemetry::recorder::TelemetryRecorder;
use crate::integrations::telemetry::state::TelemetryState;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct WorkerDay {
    pub(super) worker: String,
    pub(super) date: NaiveDate,
}

fn appointment_fields(project_ref: &str, date: NaiveDate) -> AppointmentFields {
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

/// An unread calendar must not count as empty, or the worker would be unlinked from every appointment.
async fn reconcile_worker_day(
    worker_day: &WorkerDay,
    events: Result<Vec<RawVEvent>, String>,
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
    let mut by_project: BTreeMap<Option<String>, Vec<DayliteAppointment>> = BTreeMap::new();
    for appointment in existing {
        let project_ref = appointment.project_refs().next().map(str::to_string);
        by_project.entry(project_ref).or_default().push(appointment);
    }

    let mut synced = true;
    let mut succeeded = |result: Result<(), AppError>| match result {
        Ok(()) => true,
        Err(error) => {
            synced = sync_failed(telemetry, Integration::Daylite, &error);
            false
        }
    };
    let mut duplicates = Vec::new();
    for project_ref in worker_projects(*date, &events) {
        let fields = appointment_fields(&project_ref, *date);
        let mut candidates = by_project.remove(&Some(project_ref)).unwrap_or_default();
        candidates.sort_by_key(DayliteAppointment::numeric_id);
        let mut candidates = candidates.into_iter();
        let Some(kept) = candidates.next() else {
            succeeded(appointments.create(&fields, worker).await);
            continue;
        };
        let link = (!kept.links(worker)).then_some(worker.as_str());
        succeeded(appointments.update(&kept.reference, &fields, link).await);
        duplicates.extend(candidates);
    }

    for appointment in by_project.into_values().flatten().chain(duplicates) {
        if !appointment.links(worker) {
            continue;
        }
        let others_linked = appointment.contact_refs().any(|contact| contact != worker);
        succeeded(if others_linked {
            appointments.unlink(&appointment.reference, worker).await
        } else {
            appointments.delete(&appointment.reference).await
        });
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

/// Must match `appointmentSyncFailuresEvent` in `use-appointment-sync-failures.ts`.
const APPOINTMENT_SYNC_FAILURES_EVENT: &str = "daylite-appointment-sync-failures";

#[derive(Default)]
pub(super) struct AppointmentSync {
    /// Keeps two quick writes from both creating the day's appointment.
    locks: Mutex<HashMap<NaiveDate, Arc<tokio::sync::Mutex<()>>>>,
    failed: Mutex<BTreeSet<WorkerDay>>,
}

impl AppointmentSync {
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

    /// Notifies under the lock, so outcomes finishing together reach the frontend in order.
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

    /// One after the other, because a move's target must be linked before its source is unlinked.
    pub(super) async fn run_in_order<Fut, N>(
        &self,
        worker_days: Vec<WorkerDay>,
        reconcile: impl Fn(WorkerDay) -> Fut,
        notify: &N,
    ) where
        Fut: Future<Output = bool>,
        N: Fn(Vec<String>),
    {
        for worker_day in worker_days {
            let synchronized = worker_day.clone();
            self.run(worker_day, || reconcile(synchronized), notify)
                .await;
        }
    }

    /// One at a time, because Daylite answers a burst of requests with 429.
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
        self.run_in_order(failed, reconcile, notify).await;
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

pub(super) fn schedule_appointment_sync(app: &tauri::AppHandle, worker_days: Vec<WorkerDay>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        appointment_sync()
            .run_in_order(
                worker_days,
                |worker_day| reconcile_in_app(&app, worker_day),
                &failure_notifier(&app),
            )
            .await;
    });
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
    reconcile_worker_day(&worker_day, events, &appointments, telemetry).await
}

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
    use crate::error::AppErrorCode;
    use crate::integrations::daylite::appointments::{DayliteContactLink, DayliteProjectLink};
    use crate::integrations::local_store::EmployeeSetting;
    use crate::integrations::telemetry::recorder::TelemetryCommand;
    use crate::integrations::telemetry::test_support::{enabled_recorder, recorded};
    use std::sync::atomic::{AtomicBool, Ordering};

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

    fn projects(references: &[&str]) -> BTreeSet<String> {
        references
            .iter()
            .map(|reference| reference.to_string())
            .collect()
    }

    fn existing(reference: &str, project_ref: &str, contacts: &[&str]) -> DayliteAppointment {
        DayliteAppointment {
            reference: reference.to_string(),
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

    #[test]
    fn appointments_are_titled_after_their_day_and_run_from_eight_to_four() {
        let fields = appointment_fields(NORD, day());

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

    #[derive(Default)]
    struct StubAppointments {
        existing: Vec<DayliteAppointment>,
        search_fails: bool,
        /// Project references whose create fails, and appointment references whose other requests fail.
        failing: Vec<String>,
        requests: Mutex<Vec<String>>,
    }

    impl StubAppointments {
        fn respond<T>(&self, request: String, fails: bool, value: T) -> Result<T, AppError> {
            self.requests.lock().unwrap().push(request);
            if fails {
                return Err(AppError {
                    code: AppErrorCode::ServerError,
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
        async fn search_day(&self, date: NaiveDate) -> Result<Vec<DayliteAppointment>, AppError> {
            self.respond(
                format!("search {date}"),
                self.search_fails,
                self.existing.clone(),
            )
        }

        async fn create(&self, fields: &AppointmentFields, contact: &str) -> Result<(), AppError> {
            self.respond(
                format!("create {} {contact}", fields.project_ref),
                self.fails(&fields.project_ref),
                (),
            )
        }

        async fn update(
            &self,
            reference: &str,
            _: &AppointmentFields,
            link: Option<&str>,
        ) -> Result<(), AppError> {
            let request = format!("update {reference} {}", link.unwrap_or_default());
            self.respond(request.trim_end().to_string(), self.fails(reference), ())
        }

        async fn unlink(&self, reference: &str, contact: &str) -> Result<(), AppError> {
            self.respond(
                format!("unlink {reference} {contact}"),
                self.fails(reference),
                (),
            )
        }

        async fn delete(&self, reference: &str) -> Result<(), AppError> {
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
        reconcile_worker_day(&max_today(), events, appointments, telemetry).await
    }

    /// The requests after the search of a synchronization that has to succeed.
    async fn requests_for(projects: &[&str], existing: Vec<DayliteAppointment>) -> Vec<String> {
        let appointments = StubAppointments {
            existing,
            ..Default::default()
        };
        let events = projects
            .iter()
            .map(|project| assignment(project, "08:00"))
            .collect();
        assert!(reconcile(Ok(events), &appointments, None).await);
        appointments.requests().split_off(1)
    }

    #[tokio::test]
    async fn the_worker_is_linked_to_its_projects_and_unlinked_from_the_others() {
        let without_project = DayliteAppointment {
            projects: None,
            ..existing("/v1/appointments/5002", WEST, &[MAX])
        };
        let cases = [
            (
                "a project without an appointment",
                vec![NORD],
                vec![],
                vec!["create /v1/projects/42 /v1/contacts/1"],
            ),
            (
                "a project whose appointment does not link the worker",
                vec![NORD],
                vec![existing("/v1/appointments/5000", NORD, &[ANNA])],
                vec!["update /v1/appointments/5000 /v1/contacts/1"],
            ),
            (
                "a project whose appointment links the worker",
                vec![NORD],
                vec![existing("/v1/appointments/5000", NORD, &[MAX])],
                vec!["update /v1/appointments/5000"],
            ),
            (
                "another worker's appointment",
                vec![NORD],
                vec![
                    existing("/v1/appointments/5000", NORD, &[MAX]),
                    existing("/v1/appointments/5001", SUED, &[ANNA]),
                ],
                vec!["update /v1/appointments/5000"],
            ),
            (
                "a left project whose appointment links another contact",
                vec![],
                vec![existing("/v1/appointments/5000", NORD, &[MAX, OUTSIDER])],
                vec!["unlink /v1/appointments/5000 /v1/contacts/1"],
            ),
            (
                "a left project whose appointment links only the worker",
                vec![],
                vec![existing("/v1/appointments/5000", NORD, &[MAX])],
                vec!["delete /v1/appointments/5000"],
            ),
            (
                "an appointment without a project",
                vec![],
                vec![without_project],
                vec!["delete /v1/appointments/5002"],
            ),
            (
                "duplicates, of which the worker keeps only the lowest ID",
                vec![NORD],
                vec![
                    existing("/v1/appointments/1000", NORD, &[ANNA]),
                    existing("/v1/appointments/900", NORD, &[THIRD]),
                    existing("/v1/appointments/1100", NORD, &[OUTSIDER, MAX]),
                    existing("/v1/appointments/1200", NORD, &[MAX]),
                ],
                vec![
                    "update /v1/appointments/900 /v1/contacts/1",
                    "unlink /v1/appointments/1100 /v1/contacts/1",
                    "delete /v1/appointments/1200",
                ],
            ),
        ];

        for (label, projects, existing, expected) in cases {
            assert_eq!(
                requests_for(&projects, existing).await,
                expected,
                "case: {label}"
            );
        }
    }

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
    async fn a_failed_create_still_sends_the_other_requests() {
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
    async fn the_worker_days_of_one_write_run_one_after_the_other_in_order() {
        let sync = AppointmentSync::default();
        let log = Mutex::new(Vec::new());

        sync.run_in_order(
            vec![on(ANNA, next_day()), on(MAX, day())],
            |worker_day| {
                let log = &log;
                async move {
                    log.lock().unwrap().push(("started", worker_day.clone()));
                    tokio::task::yield_now().await;
                    log.lock().unwrap().push(("finished", worker_day));
                    true
                }
            },
            &ignore,
        )
        .await;

        assert_eq!(
            *log.lock().unwrap(),
            [
                ("started", on(ANNA, next_day())),
                ("finished", on(ANNA, next_day())),
                ("started", on(MAX, day())),
                ("finished", on(MAX, day())),
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
    async fn a_retry_synchronizes_every_failed_worker_day_and_keeps_those_failing_again() {
        let sync = failing(&[on(MAX, day()), on(MAX, next_day())]).await;
        let notifications = Notifications::default();
        let retried = Mutex::new(Vec::new());

        sync.retry_failed(
            |worker_day| {
                let retried = &retried;
                async move {
                    tokio::task::yield_now().await;
                    retried.lock().unwrap().push(worker_day.clone());
                    worker_day.date == day()
                }
            },
            &notifications.notify(),
        )
        .await;

        assert_eq!(
            *retried.lock().unwrap(),
            [on(MAX, day()), on(MAX, next_day())]
        );
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
