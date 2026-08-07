use chrono::{NaiveDate, NaiveTime};
use tauri_plugin_http::reqwest;
use uuid::Uuid;

use super::super::ical::{build_ical_payload, parse_ical_events};
use super::super::slots::{full_window, plan_slot_updates, DayPlacement, DayPlan, SlotUpdate};
use super::super::types::{MoveAssignmentResult, RawVEvent};
use super::report::fetch_events_in_range;

const MAX_REALLOCATE_ATTEMPTS: u32 = 3;

pub(crate) struct CaldavSession {
    pub(crate) client: reqwest::Client,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) base_url: String,
    pub(crate) absence_urls: Vec<String>,
}

pub(crate) struct AssignmentWrite {
    pub(crate) date: String,
    pub(crate) project_ref: String,
    pub(crate) project_name: String,
    /// Requested position among the target day's assignments. `None` keeps an existing
    /// assignment where it is and appends a new one.
    pub(crate) order_index: Option<u32>,
}

fn parent_collection_url(resource_url: &str) -> &str {
    resource_url
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or(resource_url)
}

pub(crate) async fn fetch_event_by_href(
    session: &CaldavSession,
    href: &str,
) -> Result<Option<RawVEvent>, String> {
    let resource_url = resolve_href(href, &session.base_url)?;
    fetch_event(session, &resource_url).await
}

async fn fetch_event_date(
    session: &CaldavSession,
    resource_url: &str,
) -> Result<Option<String>, String> {
    Ok(fetch_event(session, resource_url)
        .await?
        .map(|event| event.dtstart))
}

/// lkr-planner writes one VEVENT per resource, so the first component is authoritative.
async fn fetch_event(
    session: &CaldavSession,
    resource_url: &str,
) -> Result<Option<RawVEvent>, String> {
    let response = session
        .client
        .get(resource_url)
        .basic_auth(&session.username, Some(&session.password))
        .send()
        .await
        .map_err(|e| format!("Einsatz konnte nicht abgerufen werden: {e}"))?;

    let status = response.status().as_u16();
    if status == 404 {
        return Ok(None);
    }
    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    let ical_text = response
        .text()
        .await
        .map_err(|e| format!("Einsatz konnte nicht gelesen werden: {e}"))?;
    let events = parse_ical_events(&ical_text)?;
    let Some(event) = events.into_iter().next() else {
        eprintln!("calendar: {resource_url} contained no readable VEVENT");
        return Ok(None);
    };
    Ok(Some(event))
}

/// Re-plans and re-PUTs the day until no PUT is rejected with 412, so a plan that raced a
/// concurrent edit is rebuilt against the day's current state instead of being retried as is.
///
/// Each PUT carries If-Match only when the day REPORT supplied an ETag, so a server that
/// omits one degrades to an unguarded write rather than blocking re-allocation.
async fn replan_day_until_settled(
    session: &CaldavSession,
    calendar_url: &str,
    date: &str,
    placement: Option<DayPlacement<'_>>,
) -> Result<(), String> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Datum: {date}"))?;

    for _ in 0..MAX_REALLOCATE_ATTEMPTS {
        let events =
            fetch_events_in_range(session, calendar_url, day, day + chrono::Duration::days(1))
                .await?;

        let updates = plan_slot_updates(&events, date, placement).updates;
        if !put_slot_updates(session, date, updates).await? {
            return Ok(());
        }
    }

    Err(format!(
        "Zeitfenster für {date} konnten wegen gleichzeitiger Änderungen nicht aktualisiert werden."
    ))
}

/// The updates target distinct resources and do not depend on each other, so they go out
/// concurrently. Returns `true` when a PUT was rejected with 412 and the day needs
/// re-planning.
async fn put_slot_updates(
    session: &CaldavSession,
    date: &str,
    updates: Vec<SlotUpdate>,
) -> Result<bool, String> {
    let requests = updates.into_iter().map(|update| async move {
        let resource_url = resolve_href(&update.href, &session.base_url)?;

        eprintln!("calendar: slot re-allocation PUT {resource_url}");

        let mut request = session
            .client
            .put(&resource_url)
            .basic_auth(&session.username, Some(&session.password))
            .header("Content-Type", "text/calendar; charset=utf-8");
        if !update.etag.is_empty() {
            request = request.header("If-Match", update.etag);
        }
        let response = request.body(update.payload).send().await.map_err(|e| {
            format!("Zeitfenster für {date} konnten nicht aktualisiert werden: {e}")
        })?;
        Ok::<u16, String>(response.status().as_u16())
    });

    let mut conflicted = false;
    for result in futures::future::join_all(requests).await {
        let status = result?;
        if status == 412 {
            conflicted = true;
            continue;
        }
        if !(200..300).contains(&status) {
            return Err(format!(
                "Zeitfenster für {date} konnten nicht aktualisiert werden: HTTP {status}"
            ));
        }
    }
    Ok(conflicted)
}

/// Failures are logged instead of returned: the primary write already succeeded, so
/// surfacing an error here would invite a retry that duplicates the event. The next write
/// on this day converges anyway.
async fn reallocate_day_best_effort(session: &CaldavSession, calendar_url: &str, date: &str) {
    if let Err(e) = replan_day_until_settled(session, calendar_url, date, None).await {
        eprintln!("calendar: re-allocation for {date} failed (converges on the next write): {e}");
    }
}

/// The returned plan carries both the pending event's own slot and the PUTs for its
/// neighbours, so one fetch serves the whole write. `None` means the day could not be
/// planned and the caller must fall back to a full re-allocation.
async fn plan_day_for_pending_write(
    session: &CaldavSession,
    calendar_url: &str,
    write: &AssignmentWrite,
    uid: &str,
) -> Option<DayPlan> {
    let date = &write.date;
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    match fetch_events_in_range(session, calendar_url, day, day + chrono::Duration::days(1)).await {
        Ok(events) => Some(plan_slot_updates(
            &events,
            date,
            Some(DayPlacement {
                uid,
                order_index: write.order_index,
                written_by_caller: true,
            }),
        )),
        Err(e) => {
            eprintln!("calendar: day fetch before write failed, using full window: {e}");
            None
        }
    }
}

/// A 412 means the plan raced a concurrent edit, so the day is re-fetched and re-planned.
async fn apply_planned_updates_best_effort(
    session: &CaldavSession,
    calendar_url: &str,
    date: &str,
    updates: Vec<SlotUpdate>,
) {
    match put_slot_updates(session, date, updates).await {
        Ok(false) => {}
        Ok(true) => reallocate_day_best_effort(session, calendar_url, date).await,
        Err(e) => {
            eprintln!(
                "calendar: re-allocation for {date} failed (converges on the next write): {e}"
            )
        }
    }
}

/// A day that could not be planned falls back to the lone-assignment case: the full window
/// and the first position.
fn placed_or_full_window(plan: &Option<DayPlan>) -> (u32, NaiveTime, NaiveTime) {
    plan.as_ref()
        .and_then(|plan| plan.placed)
        .unwrap_or_else(|| {
            let (start, end) = full_window();
            (0, start, end)
        })
}

pub(crate) async fn create_assignment_core(
    session: &CaldavSession,
    calendar_url: &str,
    write: &AssignmentWrite,
) -> Result<String, String> {
    refuse_absence_calendar(session, calendar_url, "create_assignment")?;

    let uid = Uuid::new_v4().to_string();
    let plan = plan_day_for_pending_write(session, calendar_url, write, &uid).await;
    let (order_index, slot_start, slot_end) = placed_or_full_window(&plan);
    let payload = build_ical_payload(
        &uid,
        &write.date,
        &write.project_name,
        &write.project_ref,
        slot_start,
        slot_end,
        order_index,
    );

    let base = calendar_url.trim_end_matches('/');
    let resource_url = format!("{base}/{uid}.ics");

    eprintln!("calendar: create_assignment PUT {resource_url}");

    put_ical(
        session,
        &resource_url,
        payload,
        "Einsatz konnte nicht gespeichert werden",
    )
    .await?;

    match plan {
        Some(plan) => {
            apply_planned_updates_best_effort(session, calendar_url, &write.date, plan.updates)
                .await
        }
        None => reallocate_day_best_effort(session, calendar_url, &write.date).await,
    }

    Ok(resource_url)
}

pub(crate) async fn update_assignment_core(
    session: &CaldavSession,
    href: &str,
    uid: &str,
    write: &AssignmentWrite,
) -> Result<(), String> {
    let resource_url = resolve_href(href, &session.base_url)?;

    refuse_absence_calendar(session, &resource_url, "update_assignment")?;

    let calendar_url = parent_collection_url(&resource_url);
    // Read the event's current day before the PUT overwrites it: moving an assignment to
    // another day leaves the source day needing re-allocation too.
    let (previous_date, plan) = tokio::join!(
        fetch_event_date(session, &resource_url),
        plan_day_for_pending_write(session, calendar_url, write, uid),
    );
    let previous_date = match previous_date {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "calendar: could not read event before update, skipping source-day re-allocation: {e}"
            );
            None
        }
    };

    let (order_index, slot_start, slot_end) = placed_or_full_window(&plan);
    let payload = build_ical_payload(
        uid,
        &write.date,
        &write.project_name,
        &write.project_ref,
        slot_start,
        slot_end,
        order_index,
    );

    eprintln!("calendar: update_assignment PUT {resource_url}");

    put_ical(
        session,
        &resource_url,
        payload,
        "Einsatz konnte nicht aktualisiert werden",
    )
    .await?;

    match plan {
        Some(plan) => {
            apply_planned_updates_best_effort(session, calendar_url, &write.date, plan.updates)
                .await
        }
        None => reallocate_day_best_effort(session, calendar_url, &write.date).await,
    }
    if let Some(previous) = previous_date {
        if previous != write.date {
            reallocate_day_best_effort(session, calendar_url, &previous).await;
        }
    }

    Ok(())
}

/// Moving a card within its own cell changes nothing but the day's ordering, so the event is
/// never rewritten from the payload: the day is re-sequenced and the affected events, this one
/// included, are patched in place.
pub(crate) async fn reorder_assignment_core(
    session: &CaldavSession,
    href: &str,
    uid: &str,
    date: &str,
    order_index: u32,
) -> Result<(), String> {
    let resource_url = resolve_href(href, &session.base_url)?;

    refuse_absence_calendar(session, &resource_url, "reorder_assignment")?;

    let calendar_url = parent_collection_url(&resource_url);
    let placement = DayPlacement {
        uid,
        order_index: Some(order_index),
        written_by_caller: false,
    };
    replan_day_until_settled(session, calendar_url, date, Some(placement)).await
}

pub(crate) async fn delete_assignment_core(
    session: &CaldavSession,
    href: &str,
) -> Result<(), String> {
    let resource_url = resolve_href(href, &session.base_url)?;

    refuse_absence_calendar(session, &resource_url, "delete_assignment")?;

    // Read the event's day before deleting so the remaining same-day assignments
    // can be re-allocated afterwards.
    let event_date = match fetch_event_date(session, &resource_url).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("calendar: could not read event before delete, skipping re-allocation: {e}");
            None
        }
    };

    eprintln!("calendar: delete_assignment DELETE {resource_url}");

    let response = session
        .client
        .delete(&resource_url)
        .basic_auth(&session.username, Some(&session.password))
        .send()
        .await
        .map_err(|e| format!("Einsatz konnte nicht gelöscht werden: {e}"))?;

    let status = response.status().as_u16();
    // Treat a missing event as success: delete is idempotent (no error if already absent).
    if status == 404 {
        return Ok(());
    }
    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    if let Some(date) = event_date {
        let calendar_url = parent_collection_url(&resource_url);
        reallocate_day_best_effort(session, calendar_url, &date).await;
    }

    Ok(())
}

/// A failed target create returns `Err` and leaves the source untouched; a failed
/// source delete returns `SourceDeleteFailed` for the caller to reconcile.
pub(crate) async fn move_assignment_core(
    session: &CaldavSession,
    source_href: &str,
    target_calendar_url: &str,
    write: &AssignmentWrite,
) -> Result<MoveAssignmentResult, String> {
    let new_href = create_assignment_core(session, target_calendar_url, write).await?;

    match delete_assignment_core(session, source_href).await {
        Ok(()) => Ok(MoveAssignmentResult::Moved { new_href }),
        Err(error) => {
            eprintln!(
                "calendar: move_assignment source delete failed after target create: {error}"
            );
            Ok(MoveAssignmentResult::SourceDeleteFailed {
                new_href,
                source_href: source_href.to_string(),
            })
        }
    }
}

/// CalDAV servers return root-absolute hrefs; joining one onto a `base_url` that
/// already contains a path would duplicate the path segment and produce a 404,
/// so the href is resolved against the scheme+host origin only.
pub(super) fn resolve_href(href: &str, base_url: &str) -> Result<String, String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Ok(href.to_string());
    }
    let origin =
        reqwest::Url::parse(base_url).map_err(|e| format!("Ungültige Kalender-URL: {e}"))?;
    let resolved = origin
        .join(href)
        .map_err(|e| format!("Kalender-URL konnte nicht aufgelöst werden: {e}"))?;
    Ok(resolved.to_string())
}

/// Safety guard every assignment write goes through: writes must never land in an
/// absence calendar, even if the store is misconfigured (primary == absence) or an
/// href is corrupted.
fn refuse_absence_calendar(
    session: &CaldavSession,
    target_url: &str,
    operation: &str,
) -> Result<(), String> {
    if !targets_absence_calendar(target_url, &session.absence_urls) {
        return Ok(());
    }

    eprintln!("calendar: refused {operation} write to absence calendar URL '{target_url}'");
    Err("Einsätze können nicht in einen Abwesenheitskalender geschrieben werden.".to_string())
}

async fn put_ical(
    session: &CaldavSession,
    resource_url: &str,
    payload: String,
    failure_message: &str,
) -> Result<(), String> {
    let response = session
        .client
        .put(resource_url)
        .basic_auth(&session.username, Some(&session.password))
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("{failure_message}: {e}"))?;

    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    Ok(())
}

fn targets_absence_calendar(target_url: &str, absence_urls: &[String]) -> bool {
    let target = target_url.trim_end_matches('/');
    absence_urls.iter().any(|raw| {
        let absence = raw.trim_end_matches('/');
        !absence.is_empty() && (target == absence || target.starts_with(&format!("{absence}/")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Routes are matched by method + path prefix, and every request is recorded
    /// in arrival order.
    struct TestServer {
        base_url: String,
        received: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl TestServer {
        async fn spawn(routes: Vec<(&'static str, &'static str, u16)>) -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind test server");
            let addr = listener.local_addr().expect("test server addr");
            let received: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
            let recorded = received.clone();

            tokio::spawn(async move {
                loop {
                    let Ok((mut stream, _)) = listener.accept().await else {
                        break;
                    };
                    let routes = routes.clone();
                    let recorded = recorded.clone();
                    tokio::spawn(async move {
                        let Some((method, path)) = read_request(&mut stream).await else {
                            return;
                        };
                        recorded
                            .lock()
                            .unwrap()
                            .push((method.clone(), path.clone()));
                        let status = routes
                            .iter()
                            .find(|(m, prefix, _)| *m == method && path.starts_with(prefix))
                            .map(|(_, _, status)| *status)
                            .unwrap_or(404);
                        let response = format!(
                            "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                        let _ = stream.shutdown().await;
                    });
                }
            });

            Self {
                base_url: format!("http://{addr}"),
                received,
            }
        }

        fn requests(&self) -> Vec<(String, String)> {
            self.received.lock().unwrap().clone()
        }
    }

    async fn read_request(stream: &mut tokio::net::TcpStream) -> Option<(String, String)> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        let head_end = loop {
            let read = stream.read(&mut chunk).await.ok()?;
            if read == 0 {
                return None;
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
        };

        let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
        let content_length: usize = head
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().ok())?
            })
            .unwrap_or(0);

        // Drain the body so the client can finish writing before we respond.
        let mut body_read = buffer.len() - head_end;
        while body_read < content_length {
            let read = stream.read(&mut chunk).await.ok()?;
            if read == 0 {
                break;
            }
            body_read += read;
        }

        let mut request_line = head.lines().next()?.split_whitespace();
        let method = request_line.next()?.to_string();
        let path = request_line.next()?.to_string();
        Some((method, path))
    }

    fn test_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap()
    }

    fn move_session(base_url: &str, absence_urls: Vec<String>) -> CaldavSession {
        CaldavSession {
            client: test_client(),
            username: "user".to_string(),
            password: "pass".to_string(),
            base_url: base_url.to_string(),
            absence_urls,
        }
    }

    fn move_write() -> AssignmentWrite {
        AssignmentWrite {
            date: "2026-07-08".to_string(),
            project_ref: "/v1/projects/42".to_string(),
            project_name: "Projekt Nord".to_string(),
            order_index: None,
        }
    }

    #[tokio::test]
    async fn move_assignment_core_creates_on_target_then_deletes_source() {
        let server =
            TestServer::spawn(vec![("PUT", "/target/", 201), ("DELETE", "/source/", 204)]).await;
        let target_calendar = format!("{}/target", server.base_url);

        let result = move_assignment_core(
            &move_session(&server.base_url, vec![]),
            "/source/old-uid.ics",
            &target_calendar,
            &move_write(),
        )
        .await
        .expect("full move should succeed");

        let MoveAssignmentResult::Moved { new_href } = result else {
            panic!("expected Moved, got {result:?}");
        };
        assert!(new_href.starts_with(&format!("{target_calendar}/")));
        assert!(new_href.ends_with(".ics"));

        // Slot re-allocation reads the affected days around both writes (REPORT before the
        // create, GET before the delete), so assert on the writes and their order rather
        // than on an exact request count.
        let requests = server.requests();
        let writes: Vec<_> = requests
            .iter()
            .filter(|(method, _)| method == "PUT" || method == "DELETE")
            .collect();
        assert_eq!(
            writes.len(),
            2,
            "expected exactly one PUT and one DELETE: {requests:?}"
        );
        assert_eq!(writes[0].0, "PUT");
        assert!(writes[0].1.starts_with("/target/"));
        assert_eq!(
            *writes[1],
            ("DELETE".to_string(), "/source/old-uid.ics".to_string())
        );
    }

    /// The fixed-appointment guard protects existing events, which it identifies by
    /// reading them first. Creating an assignment reads no event, so it stays unguarded.
    #[tokio::test]
    async fn create_assignment_core_writes_without_reading_an_existing_event() {
        let server = TestServer::spawn(vec![("PUT", "/target/", 201)]).await;
        let target_calendar = format!("{}/target", server.base_url);

        create_assignment_core(
            &move_session(&server.base_url, vec![]),
            &target_calendar,
            &move_write(),
        )
        .await
        .expect("create should succeed");

        let requests = server.requests();
        assert!(
            requests.iter().all(|(method, _)| method != "GET"),
            "creating an assignment must not read an event: {requests:?}"
        );
        assert!(requests.iter().any(|(method, _)| method == "PUT"));
    }

    #[tokio::test]
    async fn move_assignment_core_reports_partial_move_when_source_delete_fails() {
        let server =
            TestServer::spawn(vec![("PUT", "/target/", 201), ("DELETE", "/source/", 500)]).await;
        let target_calendar = format!("{}/target", server.base_url);

        let result = move_assignment_core(
            &move_session(&server.base_url, vec![]),
            "/source/old-uid.ics",
            &target_calendar,
            &move_write(),
        )
        .await
        .expect("partial move is not an Err");

        let MoveAssignmentResult::SourceDeleteFailed {
            new_href,
            source_href,
        } = result
        else {
            panic!("expected SourceDeleteFailed, got {result:?}");
        };
        assert!(new_href.starts_with(&format!("{target_calendar}/")));
        assert_eq!(source_href, "/source/old-uid.ics");
    }

    #[tokio::test]
    async fn move_assignment_core_leaves_source_intact_when_target_create_fails() {
        let server = TestServer::spawn(vec![("PUT", "/target/", 500)]).await;
        let target_calendar = format!("{}/target", server.base_url);

        let result = move_assignment_core(
            &move_session(&server.base_url, vec![]),
            "/source/old-uid.ics",
            &target_calendar,
            &move_write(),
        )
        .await;

        assert!(result.is_err(), "failed target create must be an Err");
        let requests = server.requests();
        assert!(
            requests.iter().all(|(method, _)| method != "DELETE"),
            "source must not be deleted when the create fails: {requests:?}"
        );
    }

    #[tokio::test]
    async fn move_assignment_core_refuses_write_into_absence_calendar() {
        let server = TestServer::spawn(vec![("PUT", "/target/", 201)]).await;
        let target_calendar = format!("{}/target", server.base_url);
        let absence_urls = vec![target_calendar.clone()];

        let result = move_assignment_core(
            &move_session(&server.base_url, absence_urls),
            "/source/old-uid.ics",
            &target_calendar,
            &move_write(),
        )
        .await;

        let error = result.expect_err("absence calendar write must be refused");
        assert!(
            error.contains("Abwesenheitskalender"),
            "expected German absence refusal, got: {error}"
        );
        assert!(
            server.requests().is_empty(),
            "no request may reach the server when the guard refuses the write"
        );
    }

    #[test]
    fn resolve_href_joins_root_absolute_path_against_server_origin() {
        let result = resolve_href(
            "/caldav/admin/emp-1/uid-1.ics",
            "https://app.zep.de/caldav/admin",
        )
        .unwrap();
        assert_eq!(result, "https://app.zep.de/caldav/admin/emp-1/uid-1.ics");
    }

    #[test]
    fn resolve_href_passes_through_absolute_url_unchanged() {
        let abs = "https://app.zep.de/caldav/admin/emp-1/uid-1.ics";
        assert_eq!(
            resolve_href(abs, "https://app.zep.de/caldav/admin").unwrap(),
            abs
        );
    }

    #[test]
    fn parent_collection_url_strips_the_resource_segment() {
        assert_eq!(
            parent_collection_url("https://app.zep.de/caldav/admin/emp-1/uid-1.ics"),
            "https://app.zep.de/caldav/admin/emp-1"
        );
    }

    use super::super::report::discover_calendar_by_name;

    const TEST_DATE: &str = "2026-05-06";

    /// The delete cleans up the event the create made, so the flow leaves the server as it
    /// found it.
    async fn run_write_path_flow(
        session: &CaldavSession,
        home_set_url: &str,
        calendar_name: &str,
    ) -> Result<(), String> {
        let calendar_url = discover_calendar_by_name(session, home_set_url, calendar_name).await?;

        let href = create_assignment_core(
            session,
            &calendar_url,
            &AssignmentWrite {
                date: TEST_DATE.to_string(),
                project_ref: "/v1/projects/42".to_string(),
                project_name: "Testprojekt".to_string(),
                order_index: None,
            },
        )
        .await?;
        if !href.ends_with(".ics") {
            return Err(format!("unexpected resource href: {href}"));
        }

        let uid = href
            .rsplit('/')
            .next()
            .and_then(|segment| segment.strip_suffix(".ics"))
            .ok_or_else(|| format!("cannot derive uid from href: {href}"))?
            .to_string();

        update_assignment_core(
            session,
            &href,
            &uid,
            &AssignmentWrite {
                date: TEST_DATE.to_string(),
                project_ref: "/v1/projects/43".to_string(),
                project_name: "Aktualisiertes Projekt".to_string(),
                order_index: None,
            },
        )
        .await?;

        delete_assignment_core(session, &href).await?;
        Ok(())
    }

    struct RadicaleServer {
        child: std::process::Child,
        dir: std::path::PathBuf,
        port: u16,
    }

    impl RadicaleServer {
        /// Panics rather than skipping on any failure, so a missing `uv`, a failed
        /// on-demand install, or a server that never starts fails the test instead of
        /// silently passing without exercising the write path.
        fn start() -> Self {
            let port = free_tcp_port();
            let dir =
                std::env::temp_dir().join(format!("radicale-it-{}-{port}", std::process::id()));
            std::fs::create_dir_all(dir.join("collections"))
                .expect("temp dir for radicale should be creatable");
            std::fs::write(dir.join("htpasswd"), "testuser:testpass\n")
                .expect("htpasswd file should be writable");
            std::fs::write(
                dir.join("config"),
                format!(
                    "[server]\nhosts = 127.0.0.1:{port}\n[auth]\ntype = htpasswd\nhtpasswd_filename = {htpasswd}\nhtpasswd_encryption = plain\n[storage]\nfilesystem_folder = {storage}\n[rights]\ntype = owner_only\n[logging]\nlevel = warning\n",
                    htpasswd = dir.join("htpasswd").display(),
                    storage = dir.join("collections").display(),
                ),
            )
            .expect("radicale config should be writable");

            // Redirected to files (not piped) so a chatty server can never block on a full
            // pipe buffer; read back into the panic message if startup fails.
            let stdout = std::fs::File::create(dir.join("stdout.log"))
                .expect("stdout log file should be creatable");
            let stderr = std::fs::File::create(dir.join("stderr.log"))
                .expect("stderr log file should be creatable");

            let mut command = std::process::Command::new("uvx");
            command
                .args(["radicale", "--config"])
                .arg(dir.join("config"))
                .stdout(stdout)
                .stderr(stderr);
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                // `uvx` forks the real Radicale interpreter rather than exec-replacing
                // itself, so it becomes a separate process; making it its own process
                // group leader lets Drop kill the whole group instead of orphaning it.
                command.process_group(0);
            }

            let child = command.spawn().unwrap_or_else(|e| {
                panic!(
                    "failed to spawn `uvx radicale`: {e}\n\
                     Install uv (https://docs.astral.sh/uv/getting-started/installation/) \
                     so `uvx` is on PATH; it fetches Radicale on demand."
                )
            });

            let mut server = RadicaleServer { child, dir, port };
            for _ in 0..100 {
                if let Some(status) = server
                    .child
                    .try_wait()
                    .expect("polling the radicale process should not fail")
                {
                    panic!(
                        "`uvx radicale` exited early with {status} before becoming ready:\n{}",
                        server.read_logs()
                    );
                }
                if std::net::TcpStream::connect(("127.0.0.1", server.port)).is_ok() {
                    return server;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            panic!(
                "radicale did not become ready on port {port} within 10s:\n{}",
                server.read_logs()
            );
        }

        fn read_logs(&self) -> String {
            let stdout = std::fs::read_to_string(self.dir.join("stdout.log")).unwrap_or_default();
            let stderr = std::fs::read_to_string(self.dir.join("stderr.log")).unwrap_or_default();
            format!("--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}")
        }

        fn base_url(&self) -> String {
            format!("http://127.0.0.1:{}", self.port)
        }

        fn home_set_url(&self) -> String {
            format!("http://127.0.0.1:{}/testuser/", self.port)
        }
    }

    impl Drop for RadicaleServer {
        fn drop(&mut self) {
            // Kill the whole process group `uvx` and the Radicale it forked belong to;
            // `child.kill()` alone only reaches the immediate `uvx` process and leaves
            // Radicale running, reparented to init.
            #[cfg(unix)]
            {
                let pgid = self.child.id();
                let _ = std::process::Command::new("kill")
                    .args(["-9", "--", &format!("-{pgid}")])
                    .status();
            }
            #[cfg(not(unix))]
            {
                let _ = self.child.kill();
            }
            let _ = self.child.wait();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn free_tcp_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    /// A client that bypasses any ambient HTTP(S) proxy so it can reach 127.0.0.1 directly.
    fn direct_client() -> reqwest::Client {
        reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    }

    /// The production client never issues MKCALENDAR, so the test seeds the calendar itself
    /// and discovery then finds it by name.
    async fn seed_radicale_calendar(server: &RadicaleServer, calid: &str, display_name: &str) {
        let url = format!("{}/testuser/{calid}/", server.base_url());
        let body = format!(
            "<C:mkcalendar xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\"><D:set><D:prop><D:displayname>{display_name}</D:displayname></D:prop></D:set></C:mkcalendar>"
        );
        let status = direct_client()
            .request(reqwest::Method::from_bytes(b"MKCALENDAR").unwrap(), &url)
            .basic_auth("testuser", Some("testpass"))
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .expect("MKCALENDAR should reach radicale")
            .status()
            .as_u16();
        assert!(
            (200..300).contains(&status),
            "MKCALENDAR failed: HTTP {status}"
        );
    }

    #[tokio::test]
    async fn caldav_write_path_against_disposable_radicale() {
        let server = RadicaleServer::start();
        seed_radicale_calendar(&server, "neuburg", "neuburg-termine").await;

        let session = CaldavSession {
            client: direct_client(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            base_url: server.base_url(),
            absence_urls: vec![],
        };

        run_write_path_flow(&session, &server.home_set_url(), "neuburg-termine")
            .await
            .expect("write path against disposable radicale should succeed");
    }

    #[test]
    fn targets_absence_calendar_matches_collection_and_resources_beneath_it() {
        let absence = vec!["https://app.zep.de/caldav/admin/emp/absence".to_string()];

        assert!(targets_absence_calendar(
            "https://app.zep.de/caldav/admin/emp/absence",
            &absence,
        ));
        assert!(targets_absence_calendar(
            "https://app.zep.de/caldav/admin/emp/absence/",
            &absence,
        ));
        assert!(targets_absence_calendar(
            "https://app.zep.de/caldav/admin/emp/absence/uid-1.ics",
            &absence,
        ));
    }

    #[test]
    fn targets_absence_calendar_allows_primary_calendar() {
        let absence = vec!["https://app.zep.de/caldav/admin/emp/absence".to_string()];

        assert!(!targets_absence_calendar(
            "https://app.zep.de/caldav/admin/emp/primary/uid-1.ics",
            &absence,
        ));
        assert!(!targets_absence_calendar(
            "https://app.zep.de/caldav/admin/emp/primary",
            &[],
        ));
    }
}
