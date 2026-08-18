use super::caldav::{fetch_event_by_href, CaldavSession};
use super::events::parse_daylite_reference;
use crate::integrations::daylite::projects::{ResolvedProject, FIXED_APPOINTMENT_CATEGORY};

pub(crate) const FIXED_APPOINTMENT_MESSAGE: &str = "Dieser Termin ist als 'Termin FIX geplant' gesperrt und kann nicht geändert, auf einen anderen Tag verschoben oder gelöscht werden.";

/// Refuses a write to an event whose Daylite project is a fixed appointment.
/// The link comes from the event, never the caller: an override decides only whether to proceed, not what counts as protected.
pub(crate) async fn refuse_protected_event(
    session: &CaldavSession,
    href: &str,
    override_protection: bool,
    lookup_project: impl AsyncFnOnce(String) -> Option<ResolvedProject>,
) -> Result<(), String> {
    if override_protection {
        return Ok(());
    }

    // A missing event cannot be protected; delete stays idempotent for one.
    let Some(event) = fetch_event_by_href(session, href).await? else {
        return Ok(());
    };

    let (project_ref, project) =
        resolve_project_link(href, &event.description, lookup_project).await;
    refuse(is_protected_event(project_ref.as_deref(), project.as_ref()))
}

/// Only the day is committed, so a move that keeps the date is allowed.
pub(crate) async fn refuse_protected_day_change(
    session: &CaldavSession,
    href: &str,
    target_date: &str,
    lookup_project: impl AsyncFnOnce(String) -> Option<ResolvedProject>,
) -> Result<(), String> {
    let Some(event) = fetch_event_by_href(session, href).await? else {
        return Ok(());
    };

    let (project_ref, project) =
        resolve_project_link(href, &event.description, lookup_project).await;
    refuse(protects_day_change(
        &event.dtstart,
        target_date,
        project_ref.as_deref(),
        project.as_ref(),
    ))
}

async fn resolve_project_link(
    href: &str,
    description: &str,
    lookup_project: impl AsyncFnOnce(String) -> Option<ResolvedProject>,
) -> (Option<String>, Option<ResolvedProject>) {
    let project_ref = parse_daylite_reference(description);
    let project = match project_ref.clone() {
        Some(reference) => lookup_project(reference).await,
        None => None,
    };
    if project_ref.is_some() && project.is_none() {
        eprintln!("calendar: project lookup failed for {href}, treating the event as unprotected");
    }
    (project_ref, project)
}

fn refuse(protected: bool) -> Result<(), String> {
    if protected {
        return Err(FIXED_APPOINTMENT_MESSAGE.to_string());
    }
    Ok(())
}

fn protects_day_change(
    event_date: &str,
    target_date: &str,
    project_ref: Option<&str>,
    project: Option<&ResolvedProject>,
) -> bool {
    event_date != target_date && is_protected_event(project_ref, project)
}

/// A bare event was not created by the planner, so it is protected too.
/// A project that cannot be resolved fails open: a broken link must not lock an event.
fn is_protected_event(project_ref: Option<&str>, project: Option<&ResolvedProject>) -> bool {
    if project_ref.is_none() {
        return true;
    }
    project.and_then(|p| p.category.as_deref()) == Some(FIXED_APPOINTMENT_CATEGORY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tauri_plugin_http::reqwest;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const FIXED_REF: Option<&str> = Some("/v1/projects/3001");

    const PROTECTED_EVENT: &str = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:uid-1\r\nSUMMARY:Projekt Nord\r\nDESCRIPTION:daylite:/v1/projects/3001\r\nDTSTART;VALUE=DATE:20260506\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    /// Serves one event to every GET and records how often it was asked.
    struct EventServer {
        base_url: String,
        requests: Arc<Mutex<usize>>,
    }

    impl EventServer {
        async fn spawn(body: &'static str) -> Self {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind test server");
            let addr = listener.local_addr().expect("test server addr");
            let requests = Arc::new(Mutex::new(0));
            let counted = requests.clone();

            tokio::spawn(async move {
                loop {
                    let Ok((mut stream, _)) = listener.accept().await else {
                        break;
                    };
                    let counted = counted.clone();
                    tokio::spawn(async move {
                        let mut chunk = [0u8; 1024];
                        if stream.read(&mut chunk).await.unwrap_or(0) == 0 {
                            return;
                        }
                        *counted.lock().unwrap() += 1;
                        let response = format!(
                            "HTTP/1.1 200 Test\r\nContent-Type: text/calendar\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                        let _ = stream.shutdown().await;
                    });
                }
            });

            Self {
                base_url: format!("http://{addr}"),
                requests,
            }
        }

        fn count(&self) -> usize {
            *self.requests.lock().unwrap()
        }
    }

    fn session(base_url: &str) -> CaldavSession {
        CaldavSession {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            username: "user".to_string(),
            password: "pass".to_string(),
            base_url: base_url.to_string(),
            absence_urls: Vec::new(),
        }
    }

    /// Records whether the guard reached the Daylite lookup, and answers with the given category.
    fn lookup(
        category: Option<&'static str>,
        calls: Arc<Mutex<usize>>,
    ) -> impl AsyncFnOnce(String) -> Option<ResolvedProject> {
        move |_reference: String| async move {
            *calls.lock().unwrap() += 1;
            Some(project(category))
        }
    }

    #[tokio::test]
    async fn an_overridden_write_skips_the_check_and_its_lookup() {
        let server = EventServer::spawn(PROTECTED_EVENT).await;
        let calls = Arc::new(Mutex::new(0));

        let result = refuse_protected_event(
            &session(&server.base_url),
            "/cal/uid-1.ics",
            true,
            lookup(Some(FIXED_APPOINTMENT_CATEGORY), calls.clone()),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(server.count(), 0);
        assert_eq!(*calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn a_write_without_an_override_is_still_refused() {
        let server = EventServer::spawn(PROTECTED_EVENT).await;
        let calls = Arc::new(Mutex::new(0));

        let result = refuse_protected_event(
            &session(&server.base_url),
            "/cal/uid-1.ics",
            false,
            lookup(Some(FIXED_APPOINTMENT_CATEGORY), calls.clone()),
        )
        .await;

        assert_eq!(result, Err(FIXED_APPOINTMENT_MESSAGE.to_string()));
        assert_eq!(server.count(), 1);
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn a_write_to_an_unprotected_event_is_allowed_without_an_override() {
        let server = EventServer::spawn(PROTECTED_EVENT).await;
        let calls = Arc::new(Mutex::new(0));

        let result = refuse_protected_event(
            &session(&server.base_url),
            "/cal/uid-1.ics",
            false,
            lookup(Some("Liefertermin bekannt"), calls.clone()),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(*calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn a_move_of_a_fixed_appointment_to_another_day_is_refused() {
        let server = EventServer::spawn(PROTECTED_EVENT).await;
        let calls = Arc::new(Mutex::new(0));

        let result = refuse_protected_day_change(
            &session(&server.base_url),
            "/cal/uid-1.ics",
            "2026-05-07",
            lookup(Some(FIXED_APPOINTMENT_CATEGORY), calls.clone()),
        )
        .await;

        assert_eq!(result, Err(FIXED_APPOINTMENT_MESSAGE.to_string()));
    }

    fn project(category: Option<&str>) -> ResolvedProject {
        ResolvedProject {
            name: "Projekt Nord".to_string(),
            status: "in_progress".to_string(),
            category: category.map(str::to_string),
        }
    }

    #[test]
    fn event_of_a_fixed_appointment_project_is_protected() {
        let resolved = project(Some(FIXED_APPOINTMENT_CATEGORY));

        assert!(is_protected_event(FIXED_REF, Some(&resolved)));
    }

    #[test]
    fn event_of_another_project_category_is_not_protected() {
        for category in [Some("Liefertermin bekannt"), None] {
            let resolved = project(category);

            assert!(!is_protected_event(FIXED_REF, Some(&resolved)));
        }
    }

    #[test]
    fn event_without_a_daylite_reference_is_protected() {
        assert!(is_protected_event(None, None));
    }

    #[test]
    fn event_whose_project_lookup_failed_is_not_protected() {
        assert!(!is_protected_event(FIXED_REF, None));
    }

    #[test]
    fn moving_a_fixed_appointment_to_another_day_is_refused() {
        let resolved = project(Some(FIXED_APPOINTMENT_CATEGORY));

        assert!(protects_day_change(
            "2026-05-06",
            "2026-05-07",
            FIXED_REF,
            Some(&resolved)
        ));
    }

    #[test]
    fn moving_a_fixed_appointment_within_its_day_is_allowed() {
        let resolved = project(Some(FIXED_APPOINTMENT_CATEGORY));

        assert!(!protects_day_change(
            "2026-05-06",
            "2026-05-06",
            FIXED_REF,
            Some(&resolved)
        ));
    }

    #[test]
    fn moving_an_unprotected_assignment_to_another_day_is_allowed() {
        let resolved = project(Some("Liefertermin bekannt"));

        assert!(!protects_day_change(
            "2026-05-06",
            "2026-05-07",
            FIXED_REF,
            Some(&resolved)
        ));
    }
}
