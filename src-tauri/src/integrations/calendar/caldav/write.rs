use chrono::NaiveDate;
use tauri_plugin_http::reqwest;
use uuid::Uuid;

use super::super::ical::{build_ical_payload, parse_ical_events};
use super::super::slots::{full_window, plan_slot_updates};
use super::report::fetch_events_in_range;
#[cfg(test)]
use crate::integrations::http_record_replay::{
    RecordReplayConfig, RecordedInteraction, RecordedRequest, RecordedResponse, VcrMode,
};

pub(crate) struct CaldavSession {
    pub(crate) client: reqwest::Client,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) base_url: String,
    pub(crate) absence_urls: Vec<String>,
    /// Record/replay hooks for the CalDAV write-path cassette test. `None` in production
    /// (and in unit tests that never hit the network); `Some` only in the VCR harness.
    #[cfg(test)]
    pub(crate) test_hooks: Option<TestHooks>,
}

#[cfg(test)]
pub(crate) struct TestHooks {
    pub(crate) record_replay: RecordReplayConfig,
    /// Pins the UID that `create_assignment_core` would otherwise randomise, so a recorded
    /// create request (whose resource path and body embed the UID) matches on replay.
    pub(crate) fixed_uid: String,
}

impl CaldavSession {
    /// Single seam for every CalDAV HTTP call: applies basic auth and, under `#[cfg(test)]`,
    /// records or replays the interaction via the cassette. Returns `(status, body)` for any
    /// HTTP response; `Err` only on a transport or cassette failure. Callers keep their own
    /// status handling and wrap the error with a caller-specific German message.
    pub(super) async fn send(
        &self,
        method: &str,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&str>,
    ) -> Result<(u16, String), String> {
        #[cfg(test)]
        if let Some(hooks) = &self.test_hooks {
            if hooks.record_replay.mode() == VcrMode::Replay {
                let recorded = to_recorded_request(method, url, body);
                let response = hooks
                    .record_replay
                    .replay(&recorded)
                    .map_err(|e| format!("Kassette konnte nicht gelesen werden: {e}"))?
                    .ok_or_else(|| {
                        format!("Keine Kassetten-Interaktion für {method} {}", recorded.path)
                    })?;
                return Ok((response.status, response.body));
            }
        }

        let reqwest_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| format!("Ungültige HTTP-Methode {method}: {e}"))?;
        let mut request = self
            .client
            .request(reqwest_method, url)
            .basic_auth(&self.username, Some(&self.password));
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if let Some(body) = body {
            request = request.body(body.to_string());
        }

        let response = request.send().await.map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let text = response.text().await.map_err(|e| e.to_string())?;

        #[cfg(test)]
        if let Some(hooks) = &self.test_hooks {
            if hooks.record_replay.mode() == VcrMode::Record {
                hooks
                    .record_replay
                    .record(RecordedInteraction {
                        request: to_recorded_request(method, url, body),
                        response: RecordedResponse {
                            status,
                            body: text.clone(),
                        },
                    })
                    .map_err(|e| format!("Kassette konnte nicht gespeichert werden: {e}"))?;
            }
        }

        Ok((status, text))
    }

    /// The UID a create should use: pinned in the VCR harness, random otherwise.
    fn next_uid(&self) -> String {
        #[cfg(test)]
        if let Some(hooks) = &self.test_hooks {
            return hooks.fixed_uid.clone();
        }
        Uuid::new_v4().to_string()
    }
}

/// Builds the cassette match key for a CalDAV request. The path is stored host-agnostically
/// (origin stripped) so cassettes never leak the server, and `DTSTAMP` lines are dropped
/// from iCal bodies because they carry the wall-clock time of the write and would otherwise
/// differ between record and replay.
#[cfg(test)]
fn to_recorded_request(method: &str, url: &str, body: Option<&str>) -> RecordedRequest {
    let path = reqwest::Url::parse(url)
        .map(|parsed| parsed.path().to_string())
        .unwrap_or_else(|_| url.to_string());
    RecordedRequest {
        method: method.to_string(),
        path,
        query: Vec::new(),
        body: body.map(|b| serde_json::Value::String(normalize_body(b))),
    }
}

#[cfg(test)]
fn normalize_body(body: &str) -> String {
    body.lines()
        .filter(|line| !line.starts_with("DTSTAMP:"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) struct AssignmentWrite {
    pub(crate) date: String,
    pub(crate) project_ref: String,
    pub(crate) project_name: String,
}

fn parent_collection_url(resource_url: &str) -> &str {
    resource_url
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or(resource_url)
}

/// Fetches a single event resource and returns its DTSTART date (`yyyy-MM-dd`).
/// Returns `Ok(None)` when the resource does not exist (404) or contains no event.
/// lkr-planner writes one VEVENT per resource, so the first component is authoritative.
async fn fetch_event_date(
    session: &CaldavSession,
    resource_url: &str,
) -> Result<Option<String>, String> {
    let (status, ical_text) = session
        .send("GET", resource_url, &[], None)
        .await
        .map_err(|e| format!("Einsatz konnte nicht abgerufen werden: {e}"))?;

    if status == 404 {
        return Ok(None);
    }
    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    let events = parse_ical_events(&ical_text)?;
    Ok(events.into_iter().next().map(|event| event.dtstart))
}

/// Re-allocates the 08:00-16:00 window across all lkr-planner assignments on `date`
/// and PUTs every event whose slot changed. Bare, absence, and holiday events are
/// never touched (see `plan_slot_updates`). Each PUT is guarded with If-Match on the
/// ETag from the day REPORT, when the server provided one, so a concurrent edit is
/// never clobbered in that case; on a 412 the day is re-fetched and re-planned.
async fn reallocate_day(
    session: &CaldavSession,
    calendar_url: &str,
    date: &str,
) -> Result<(), String> {
    const MAX_REALLOCATE_ATTEMPTS: u32 = 3;

    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Datum: {date}"))?;

    for _ in 0..MAX_REALLOCATE_ATTEMPTS {
        let events =
            fetch_events_in_range(session, calendar_url, day, day + chrono::Duration::days(1))
                .await?;

        let mut conflicted = false;
        for update in plan_slot_updates(&events, date, None).updates {
            let resource_url = resolve_href(&update.href, calendar_url)?;

            eprintln!("calendar: reallocate_day PUT {resource_url}");

            let mut headers: Vec<(&str, &str)> =
                vec![("Content-Type", "text/calendar; charset=utf-8")];
            if !update.etag.is_empty() {
                headers.push(("If-Match", update.etag.as_str()));
            }
            let (status, _) = session
                .send(
                    "PUT",
                    &resource_url,
                    &headers,
                    Some(update.payload.as_str()),
                )
                .await
                .map_err(|e| {
                    format!("Zeitfenster für {date} konnten nicht aktualisiert werden: {e}")
                })?;

            if status == 412 {
                // The event changed between REPORT and PUT: re-fetch and re-plan the day.
                conflicted = true;
                break;
            }
            if !(200..300).contains(&status) {
                return Err(format!(
                    "Zeitfenster für {date} konnten nicht aktualisiert werden: HTTP {status}"
                ));
            }
        }
        if !conflicted {
            return Ok(());
        }
    }

    Err(format!(
        "Zeitfenster für {date} konnten wegen gleichzeitiger Änderungen nicht aktualisiert werden."
    ))
}

/// Runs day re-allocation after the primary write already succeeded.
/// Failures are logged instead of returned: failing the whole command would make the
/// caller believe the create/update/delete itself failed (and retrying a create would
/// duplicate the event), while the next write on this day converges anyway.
async fn reallocate_day_best_effort(session: &CaldavSession, calendar_url: &str, date: &str) {
    if let Err(e) = reallocate_day(session, calendar_url, date).await {
        eprintln!("calendar: re-allocation for {date} failed (converges on the next write): {e}");
    }
}

/// Fetches the day's events and returns the slot the event `uid` will occupy once
/// written, so create/update can write the event once, directly in its final slot.
/// Falls back to the full window when the fetch fails; re-allocation converges later.
async fn slot_for_pending_write(
    session: &CaldavSession,
    calendar_url: &str,
    date: &str,
    uid: &str,
) -> (chrono::NaiveTime, chrono::NaiveTime) {
    let Ok(day) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return full_window();
    };
    match fetch_events_in_range(session, calendar_url, day, day + chrono::Duration::days(1)).await {
        Ok(events) => plan_slot_updates(&events, date, Some(uid))
            .extra_slot
            .unwrap_or_else(full_window),
        Err(e) => {
            eprintln!("calendar: day fetch before write failed, using full window: {e}");
            full_window()
        }
    }
}

pub(crate) async fn create_assignment_core(
    session: &CaldavSession,
    calendar_url: &str,
    write: &AssignmentWrite,
) -> Result<String, String> {
    if targets_absence_calendar(calendar_url, &session.absence_urls) {
        eprintln!(
            "calendar: refused create_assignment write to absence calendar URL '{calendar_url}'"
        );
        return Err(
            "Einsätze können nicht in einen Abwesenheitskalender geschrieben werden.".to_string(),
        );
    }

    let uid = session.next_uid();
    let (slot_start, slot_end) =
        slot_for_pending_write(session, calendar_url, &write.date, &uid).await;
    let payload = build_ical_payload(
        &uid,
        &write.date,
        &write.project_name,
        &write.project_ref,
        slot_start,
        slot_end,
    );

    let base = calendar_url.trim_end_matches('/');
    let resource_url = format!("{base}/{uid}.ics");

    eprintln!("calendar: create_assignment PUT {resource_url}");

    let (status, _) = session
        .send(
            "PUT",
            &resource_url,
            &[("Content-Type", "text/calendar; charset=utf-8")],
            Some(payload.as_str()),
        )
        .await
        .map_err(|e| format!("Einsatz konnte nicht gespeichert werden: {e}"))?;

    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    reallocate_day_best_effort(session, calendar_url, &write.date).await;

    Ok(resource_url)
}

pub(crate) async fn update_assignment_core(
    session: &CaldavSession,
    href: &str,
    uid: &str,
    write: &AssignmentWrite,
) -> Result<(), String> {
    let resource_url = resolve_href(href, &session.base_url)?;

    if targets_absence_calendar(&resource_url, &session.absence_urls) {
        eprintln!(
            "calendar: refused update_assignment write to absence calendar URL '{resource_url}'"
        );
        return Err(
            "Einsätze können nicht in einen Abwesenheitskalender geschrieben werden.".to_string(),
        );
    }

    // Read the event's current day before overwriting it: when the update moves the
    // assignment to another day, the source day must be re-allocated as well.
    let previous_date = match fetch_event_date(session, &resource_url).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "calendar: could not read event before update, skipping source-day re-allocation: {e}"
            );
            None
        }
    };

    let calendar_url = parent_collection_url(&resource_url);
    let (slot_start, slot_end) =
        slot_for_pending_write(session, calendar_url, &write.date, uid).await;
    let payload = build_ical_payload(
        uid,
        &write.date,
        &write.project_name,
        &write.project_ref,
        slot_start,
        slot_end,
    );

    eprintln!("calendar: update_assignment PUT {resource_url}");

    let (status, _) = session
        .send(
            "PUT",
            &resource_url,
            &[("Content-Type", "text/calendar; charset=utf-8")],
            Some(payload.as_str()),
        )
        .await
        .map_err(|e| format!("Einsatz konnte nicht aktualisiert werden: {e}"))?;

    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    reallocate_day_best_effort(session, calendar_url, &write.date).await;
    if let Some(previous) = previous_date {
        if previous != write.date {
            reallocate_day_best_effort(session, calendar_url, &previous).await;
        }
    }

    Ok(())
}

pub(crate) async fn delete_assignment_core(
    session: &CaldavSession,
    href: &str,
) -> Result<(), String> {
    let resource_url = resolve_href(href, &session.base_url)?;

    if targets_absence_calendar(&resource_url, &session.absence_urls) {
        eprintln!(
            "calendar: refused delete_assignment write to absence calendar URL '{resource_url}'"
        );
        return Err(
            "Einsätze können nicht in einen Abwesenheitskalender geschrieben werden.".to_string(),
        );
    }

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

    let (status, _) = session
        .send("DELETE", &resource_url, &[], None)
        .await
        .map_err(|e| format!("Einsatz konnte nicht gelöscht werden: {e}"))?;

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

/// Safety guard: assignment writes must never land in an absence calendar, even
/// if the store is misconfigured (primary == absence) or an href is corrupted.
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
    use std::time::Duration;

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

    // ── CalDAV write-path VCR harness ──
    //
    // The full write path (discover a calendar, create an assignment, update it, delete it)
    // is exercised end-to-end over the transport seam against a recorded cassette. The
    // committed cassette uses host-agnostic example paths and is produced deterministically
    // by `generate_caldav_write_path_cassette`. `record_caldav_write_path_cassette` runs the
    // same flow against a live server for verification and cleans up after itself.

    use super::super::report::{build_report_body, discover_calendar_by_name, PROPFIND_BODY};
    use crate::integrations::http_record_replay::{
        RecordReplayConfig, RecordedInteraction, RecordedResponse, VcrMode,
    };

    const CASSETTE_FILE: &str = "caldav-write-path.json";
    const CALENDAR_NAME: &str = "Testkalender";
    const FIXED_UID: &str = "vcr-fixed-uid-0001";
    const TEST_DATE: &str = "2026-05-06";
    const BASE_URL: &str = "https://caldav.example";
    const HOME_SET_URL: &str = "https://caldav.example/dav/";
    const CALENDAR_URL: &str = "https://caldav.example/dav/testkalender/";
    const CALENDAR_URL_NO_SLASH: &str = "https://caldav.example/dav/testkalender";
    const RESOURCE_URL: &str = "https://caldav.example/dav/testkalender/vcr-fixed-uid-0001.ics";

    fn cassette_path_for_calendar_test(file_name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/cassettes")
            .join(file_name)
    }

    fn vcr_session(
        base_url: &str,
        username: &str,
        password: &str,
        cassette_file: &str,
        mode: VcrMode,
    ) -> CaldavSession {
        CaldavSession {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            username: username.to_string(),
            password: password.to_string(),
            base_url: base_url.to_string(),
            absence_urls: vec![],
            test_hooks: Some(TestHooks {
                record_replay: RecordReplayConfig::new(
                    cassette_path_for_calendar_test(cassette_file),
                    mode,
                ),
                fixed_uid: FIXED_UID.to_string(),
            }),
        }
    }

    /// Drives discover -> create -> update -> delete over the seam. The delete cleans up the
    /// event the create made, so a live recording run leaves the server as it found it.
    async fn run_write_path_flow(
        session: &CaldavSession,
        home_set_url: &str,
    ) -> Result<(), String> {
        let calendar_url = discover_calendar_by_name(session, home_set_url, CALENDAR_NAME).await?;

        let href = create_assignment_core(
            session,
            &calendar_url,
            &AssignmentWrite {
                date: TEST_DATE.to_string(),
                project_ref: "/v1/projects/42".to_string(),
                project_name: "Testprojekt".to_string(),
            },
        )
        .await?;
        if !href.ends_with(".ics") {
            return Err(format!("unexpected resource href: {href}"));
        }

        update_assignment_core(
            session,
            &href,
            FIXED_UID,
            &AssignmentWrite {
                date: TEST_DATE.to_string(),
                project_ref: "/v1/projects/43".to_string(),
                project_name: "Aktualisiertes Projekt".to_string(),
            },
        )
        .await?;

        delete_assignment_core(session, &href).await?;
        Ok(())
    }

    #[tokio::test]
    async fn caldav_write_path_replays_cassette() {
        let session = vcr_session(
            BASE_URL,
            "vcr-user",
            "vcr-pass",
            CASSETTE_FILE,
            VcrMode::Replay,
        );
        run_write_path_flow(&session, HOME_SET_URL)
            .await
            .expect("cassette replay of the CalDAV write path should succeed");
    }

    // Regenerates the committed cassette from the exact requests the flow issues (built via the
    // real body builders, so the match keys can never drift) paired with representative CalDAV
    // responses. Run with `--ignored` after changing the flow or the builders.
    #[test]
    #[ignore = "regenerates the committed cassette fixture"]
    fn generate_caldav_write_path_cassette() {
        let (window_start, window_end) = full_window();
        let report_body = build_report_body("20260506T000000Z", "20260507T000000Z");
        let create_payload = build_ical_payload(
            FIXED_UID,
            TEST_DATE,
            "Testprojekt",
            "/v1/projects/42",
            window_start,
            window_end,
        );
        let update_payload = build_ical_payload(
            FIXED_UID,
            TEST_DATE,
            "Aktualisiertes Projekt",
            "/v1/projects/43",
            window_start,
            window_end,
        );

        let interactions = vec![
            (
                "PROPFIND",
                HOME_SET_URL,
                Some(PROPFIND_BODY),
                207,
                PROPFIND_RESPONSE,
            ),
            (
                "REPORT",
                CALENDAR_URL,
                Some(report_body.as_str()),
                207,
                REPORT_ONE_EVENT_RESPONSE,
            ),
            ("PUT", RESOURCE_URL, Some(create_payload.as_str()), 201, ""),
            ("GET", RESOURCE_URL, None, 200, GET_EVENT_RESPONSE),
            (
                "REPORT",
                CALENDAR_URL_NO_SLASH,
                Some(report_body.as_str()),
                207,
                REPORT_ONE_EVENT_RESPONSE,
            ),
            ("PUT", RESOURCE_URL, Some(update_payload.as_str()), 204, ""),
            ("DELETE", RESOURCE_URL, None, 204, ""),
        ];

        let config = RecordReplayConfig::new(
            cassette_path_for_calendar_test(CASSETTE_FILE),
            VcrMode::Record,
        );
        // Start from an empty cassette so removed interactions do not linger.
        let _ = std::fs::remove_file(cassette_path_for_calendar_test(CASSETTE_FILE));
        for (method, url, body, status, response_body) in interactions {
            config
                .record(RecordedInteraction {
                    request: to_recorded_request(method, url, body),
                    response: RecordedResponse {
                        status,
                        body: response_body.to_string(),
                    },
                })
                .expect("recording a cassette interaction should succeed");
        }
    }

    // Live verification against a real server. Requires a disposable calendar named
    // "Testkalender" (discovery matches by display name). Writes a local, git-ignored
    // cassette and cleans up the event it creates.
    #[tokio::test]
    #[ignore = "VCR: set CALDAV_URL (home set) + CALDAV_USER + CALDAV_PASS and a calendar named Testkalender"]
    async fn record_caldav_write_path_cassette() {
        let home_set_url = std::env::var("CALDAV_URL").expect("CALDAV_URL");
        let username = std::env::var("CALDAV_USER").expect("CALDAV_USER");
        let password = std::env::var("CALDAV_PASS").expect("CALDAV_PASS");

        let session = vcr_session(
            &home_set_url,
            &username,
            &password,
            "caldav-write-path.local.json",
            VcrMode::Record,
        );
        run_write_path_flow(&session, &home_set_url)
            .await
            .expect("live recording of the CalDAV write path should succeed");
    }

    const PROPFIND_RESPONSE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/dav/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Home</d:displayname>
      <d:resourcetype><d:collection/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/testkalender/</d:href>
    <d:propstat><d:prop>
      <d:displayname>Testkalender</d:displayname>
      <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
    </d:prop></d:propstat>
  </d:response>
</d:multistatus>"#;

    const REPORT_ONE_EVENT_RESPONSE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/dav/testkalender/vcr-fixed-uid-0001.ics</d:href>
    <d:propstat><d:prop>
      <d:getetag>"etag-vcr-1"</d:getetag>
      <c:calendar-data>BEGIN:VCALENDAR
BEGIN:VEVENT
UID:vcr-fixed-uid-0001
DTSTART:20260506T080000
DTEND:20260506T160000
SUMMARY:Testprojekt
DESCRIPTION:daylite:/v1/projects/42
END:VEVENT
END:VCALENDAR
</c:calendar-data>
    </d:prop></d:propstat>
  </d:response>
</d:multistatus>"#;

    const GET_EVENT_RESPONSE: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:vcr-fixed-uid-0001\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nSUMMARY:Testprojekt\r\nDESCRIPTION:daylite:/v1/projects/42\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

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
