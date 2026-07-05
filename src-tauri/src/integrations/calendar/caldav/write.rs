use tauri_plugin_http::reqwest;
use uuid::Uuid;

use super::super::ical::build_ical_payload;
use super::super::types::MoveAssignmentResult;

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
    let payload = build_ical_payload(&uid, &write.date, &write.project_name, &write.project_ref);

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

    let payload = build_ical_payload(uid, &write.date, &write.project_name, &write.project_ref);

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

    Ok(())
}

/// Moves an assignment to another calendar: creates the VEVENT on the target calendar
/// first, then deletes the source. A failed target create returns `Err` and leaves the
/// source untouched; a failed source delete returns `SourceDeleteFailed` so the caller
/// can reconcile the duplicate instead of silently keeping it.
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
fn resolve_href(href: &str, base_url: &str) -> Result<String, String> {
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
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Minimal HTTP server for exercising the CalDAV write cores without a live server.
    /// Routes are matched by method + path prefix; every request is recorded so tests
    /// can assert which writes happened and in which order.
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

    /// Reads one HTTP request (head + body per Content-Length) and returns method and path.
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

        let requests = server.requests();
        assert_eq!(
            requests.len(),
            2,
            "expected exactly PUT then DELETE: {requests:?}"
        );
        assert_eq!(requests[0].0, "PUT");
        assert!(requests[0].1.starts_with("/target/"));
        assert_eq!(
            requests[1],
            ("DELETE".to_string(), "/source/old-uid.ics".to_string())
        );
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

    #[tokio::test]
    #[ignore = "VCR: requires live CalDAV server credentials"]
    async fn create_assignment_core_sends_put_and_returns_href() {
        // To record: set CALDAV_URL, CALDAV_USER, CALDAV_PASS env vars and run with --ignored.
        let calendar_url = std::env::var("CALDAV_URL").expect("CALDAV_URL");
        let username = std::env::var("CALDAV_USER").expect("CALDAV_USER");
        let password = std::env::var("CALDAV_PASS").expect("CALDAV_PASS");

        let session = CaldavSession {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            username,
            password,
            base_url: calendar_url.clone(),
            absence_urls: vec![],
        };

        let href = create_assignment_core(
            &session,
            &calendar_url,
            &AssignmentWrite {
                date: "2026-05-06".to_string(),
                project_ref: "/v1/projects/42".to_string(),
                project_name: "Testprojekt".to_string(),
            },
        )
        .await
        .expect("create_assignment_core should succeed");

        assert!(href.starts_with(&calendar_url.trim_end_matches('/').to_string()));
        assert!(href.ends_with(".ics"));
    }

    #[tokio::test]
    #[ignore = "VCR: requires live CalDAV server credentials"]
    async fn update_assignment_core_sends_put_to_stored_href() {
        let base_url = std::env::var("CALDAV_BASE_URL").expect("CALDAV_BASE_URL");
        let href = std::env::var("CALDAV_HREF").expect("CALDAV_HREF");
        let uid = std::env::var("CALDAV_UID").expect("CALDAV_UID");
        let username = std::env::var("CALDAV_USER").expect("CALDAV_USER");
        let password = std::env::var("CALDAV_PASS").expect("CALDAV_PASS");

        let session = CaldavSession {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            username,
            password,
            base_url,
            absence_urls: vec![],
        };

        update_assignment_core(
            &session,
            &href,
            &uid,
            &AssignmentWrite {
                date: "2026-05-07".to_string(),
                project_ref: "/v1/projects/42".to_string(),
                project_name: "Aktualisiertes Projekt".to_string(),
            },
        )
        .await
        .expect("update_assignment_core should succeed");
    }

    #[tokio::test]
    #[ignore = "VCR: requires live CalDAV server credentials"]
    async fn delete_assignment_core_sends_delete_to_stored_href() {
        let base_url = std::env::var("CALDAV_BASE_URL").expect("CALDAV_BASE_URL");
        let href = std::env::var("CALDAV_HREF").expect("CALDAV_HREF");
        let username = std::env::var("CALDAV_USER").expect("CALDAV_USER");
        let password = std::env::var("CALDAV_PASS").expect("CALDAV_PASS");

        let session = CaldavSession {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            username,
            password,
            base_url,
            absence_urls: vec![],
        };

        delete_assignment_core(&session, &href)
            .await
            .expect("delete_assignment_core should succeed");
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
