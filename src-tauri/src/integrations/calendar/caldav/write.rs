use chrono::NaiveDate;
use tauri_plugin_http::reqwest;
use uuid::Uuid;

use super::super::ical::{build_ical_payload, parse_ical_events};
use super::super::slots::{full_window, plan_slot_updates};
use super::report::fetch_events_in_range;

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

            let status = response.status().as_u16();
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

    let uid = Uuid::new_v4().to_string();
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

    let response = session
        .client
        .put(&resource_url)
        .basic_auth(&session.username, Some(&session.password))
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("Einsatz konnte nicht gespeichert werden: {e}"))?;

    let status = response.status().as_u16();
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

    let response = session
        .client
        .put(&resource_url)
        .basic_auth(&session.username, Some(&session.password))
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(payload)
        .send()
        .await
        .map_err(|e| format!("Einsatz konnte nicht aktualisiert werden: {e}"))?;

    let status = response.status().as_u16();
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

    // ── Disposable Radicale server integration test ──
    //
    // Runs the full write path over real HTTP against a throwaway Radicale CalDAV server,
    // with no production credentials. Skipped when Radicale is not installed; CI installs it
    // (see the README). Validates real CalDAV request/response handling end-to-end.

    use super::super::report::discover_calendar_by_name;

    const TEST_DATE: &str = "2026-05-06";

    /// Drives discover -> create -> update -> delete. The delete cleans up the event the
    /// create made, so the flow leaves the server as it found it.
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
        /// Returns `None` when Radicale is not importable, so the test can skip cleanly.
        fn start() -> Option<Self> {
            let available = std::process::Command::new("python3")
                .args(["-c", "import radicale"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !available {
                return None;
            }

            let port = free_tcp_port();
            let dir =
                std::env::temp_dir().join(format!("radicale-it-{}-{port}", std::process::id()));
            std::fs::create_dir_all(dir.join("collections")).unwrap();
            std::fs::write(dir.join("htpasswd"), "testuser:testpass\n").unwrap();
            std::fs::write(
                dir.join("config"),
                format!(
                    "[server]\nhosts = 127.0.0.1:{port}\n[auth]\ntype = htpasswd\nhtpasswd_filename = {htpasswd}\nhtpasswd_encryption = plain\n[storage]\nfilesystem_folder = {storage}\n[rights]\ntype = owner_only\n[logging]\nlevel = warning\n",
                    htpasswd = dir.join("htpasswd").display(),
                    storage = dir.join("collections").display(),
                ),
            )
            .unwrap();

            let child = std::process::Command::new("python3")
                .args(["-m", "radicale", "--config"])
                .arg(dir.join("config"))
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("radicale should spawn");

            let server = RadicaleServer { child, dir, port };
            for _ in 0..100 {
                if std::net::TcpStream::connect(("127.0.0.1", server.port)).is_ok() {
                    return Some(server);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            panic!("radicale did not become ready on port {port}");
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
            let _ = self.child.kill();
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

    /// Creates a calendar collection with a display name. The production client never issues
    /// MKCALENDAR, so the test seeds the calendar directly, then discovery finds it by name.
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
        let Some(server) = RadicaleServer::start() else {
            eprintln!(
                "skipping caldav_write_path_against_disposable_radicale: radicale not installed"
            );
            return;
        };
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
