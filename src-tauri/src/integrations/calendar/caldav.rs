use chrono::NaiveDate;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::Method;
use uuid::Uuid;

use super::ical::{build_ical_payload, parse_ical_events};
use super::types::RawVEvent;

// ── CalDAV fetch ──────────────────────────────────────────────────────────────

pub(super) async fn fetch_calendar_events(
    client: &reqwest::Client,
    calendar_url: &str,
    username: &str,
    password: &str,
    week_start: NaiveDate,
) -> Result<Vec<RawVEvent>, String> {
    let week_end = week_start + chrono::Duration::days(7);
    let start_str = week_start.format("%Y%m%dT000000Z").to_string();
    let end_str = week_end.format("%Y%m%dT000000Z").to_string();

    let body = build_report_body(&start_str, &end_str);

    let response = client
        .request(
            Method::from_bytes(b"REPORT").expect("REPORT is a valid HTTP method"),
            calendar_url,
        )
        .basic_auth(username, Some(password))
        .header("Depth", "1")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Kalender konnte nicht abgerufen werden: {e}"))?;

    let status = response.status().as_u16();
    if status == 401 {
        return Err("Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen.".to_string());
    }
    if !(200..300).contains(&status) {
        return Err(format!("CalDAV-Server antwortete mit HTTP {status}"));
    }

    let xml_text = response
        .text()
        .await
        .map_err(|e| format!("Kalenderantwort konnte nicht gelesen werden: {e}"))?;

    parse_caldav_report(&xml_text)
        .map_err(|e| format!("Kalenderantwort konnte nicht verarbeitet werden: {e}"))
}

fn build_report_body(start: &str, end: &str) -> String {
    // Timestamps must be in the form YYYYMMDDTHHMMSSz (e.g. "20260428T000000Z").
    // They come from chrono::format so this invariant holds unless the format string changes.
    debug_assert!(
        start.len() == 16 && end.len() == 16,
        "CalDAV timestamp must be 16 chars: got start={start:?} end={end:?}"
    );
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VEVENT">
        <c:time-range start="{start}" end="{end}"/>
      </c:comp-filter>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#
    )
}

/// Parses a CalDAV REPORT XML response and extracts VEVENT entries from each calendar-data element.
/// Populates `href` on each `RawVEvent` by walking up to the enclosing `d:response` ancestor
/// and reading its `d:href` child element.
fn parse_caldav_report(xml_text: &str) -> Result<Vec<RawVEvent>, String> {
    let doc = roxmltree::Document::parse(xml_text)
        .map_err(|e| format!("XML konnte nicht geparst werden: {e}"))?;

    let mut events = Vec::new();
    for node in doc.descendants() {
        let is_caldav = node.has_tag_name(("urn:ietf:params:xml:ns:caldav", "calendar-data"));
        let is_bare = !is_caldav && node.tag_name().name() == "calendar-data";
        if is_caldav || is_bare {
            if let Some(text) = node.text() {
                let href = node
                    .ancestors()
                    .find(|a| a.has_tag_name(("DAV:", "response")))
                    .and_then(|response| {
                        response
                            .children()
                            .find(|c| c.has_tag_name(("DAV:", "href")))
                            .and_then(|h| h.text())
                    })
                    .unwrap_or("")
                    .to_string();

                let mut parsed = parse_ical_events(text)?;
                for event in &mut parsed {
                    event.href = href.clone();
                }
                events.extend(parsed);
            }
        }
    }

    Ok(events)
}

// ── CalDAV write cores ────────────────────────────────────────────────────────

pub(crate) async fn create_assignment_core(
    client: &reqwest::Client,
    calendar_url: &str,
    username: &str,
    password: &str,
    date: &str,
    project_ref: &str,
    project_name: &str,
) -> Result<String, String> {
    let uid = Uuid::new_v4().to_string();
    let payload = build_ical_payload(&uid, date, project_name, project_ref);

    let base = calendar_url.trim_end_matches('/');
    let resource_url = format!("{base}/{uid}.ics");

    let response = client
        .put(&resource_url)
        .basic_auth(username, Some(password))
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn update_assignment_core(
    client: &reqwest::Client,
    href: &str,
    base_url: &str,
    uid: &str,
    username: &str,
    password: &str,
    date: &str,
    project_ref: &str,
    project_name: &str,
) -> Result<(), String> {
    let resource_url = if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("{}{}", base_url.trim_end_matches('/'), href)
    };

    let payload = build_ical_payload(uid, date, project_name, project_ref);

    let response = client
        .put(&resource_url)
        .basic_auth(username, Some(password))
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
    client: &reqwest::Client,
    href: &str,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let resource_url = if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("{}{}", base_url.trim_end_matches('/'), href)
    };

    let response = client
        .delete(&resource_url)
        .basic_auth(username, Some(password))
        .send()
        .await
        .map_err(|e| format!("Einsatz konnte nicht gelöscht werden: {e}"))?;

    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(format!("Kalenderserver antwortete mit HTTP {status}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ── Resource URL (href) capture ──

    #[test]
    fn parse_caldav_report_returns_href_with_each_event() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/calendars/user/calendar/event1.ics</d:href>
    <d:propstat>
      <d:prop>
        <c:calendar-data>BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test-uid-1
SUMMARY:Projekt Nord
DTSTART;VALUE=DATE:20260505
END:VEVENT
END:VCALENDAR
</c:calendar-data>
      </d:prop>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

        let events = parse_caldav_report(xml).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].uid, "test-uid-1");
        assert_eq!(events[0].href, "/calendars/user/calendar/event1.ics");
    }

    #[test]
    fn parse_caldav_report_returns_correct_href_per_event_when_multiple_responses() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/calendars/user/calendar/ev1.ics</d:href>
    <d:propstat>
      <d:prop>
        <c:calendar-data>BEGIN:VCALENDAR
BEGIN:VEVENT
UID:uid-1
SUMMARY:Eins
DTSTART;VALUE=DATE:20260505
END:VEVENT
END:VCALENDAR
</c:calendar-data>
      </d:prop>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/calendars/user/calendar/ev2.ics</d:href>
    <d:propstat>
      <d:prop>
        <c:calendar-data>BEGIN:VCALENDAR
BEGIN:VEVENT
UID:uid-2
SUMMARY:Zwei
DTSTART;VALUE=DATE:20260506
END:VEVENT
END:VCALENDAR
</c:calendar-data>
      </d:prop>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

        let events = parse_caldav_report(xml).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "uid-1");
        assert_eq!(events[0].href, "/calendars/user/calendar/ev1.ics");
        assert_eq!(events[1].uid, "uid-2");
        assert_eq!(events[1].href, "/calendars/user/calendar/ev2.ics");
    }

    #[tokio::test]
    #[ignore = "VCR: requires live CalDAV server credentials"]
    async fn create_assignment_core_sends_put_and_returns_href() {
        // To record: set CALDAV_URL, CALDAV_USER, CALDAV_PASS env vars and run with --ignored.
        let calendar_url = std::env::var("CALDAV_URL").expect("CALDAV_URL");
        let username = std::env::var("CALDAV_USER").expect("CALDAV_USER");
        let password = std::env::var("CALDAV_PASS").expect("CALDAV_PASS");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        let href = create_assignment_core(
            &client,
            &calendar_url,
            &username,
            &password,
            "2026-05-06",
            "/v1/projects/42",
            "Testprojekt",
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

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        update_assignment_core(
            &client,
            &href,
            &base_url,
            &uid,
            &username,
            &password,
            "2026-05-07",
            "/v1/projects/42",
            "Aktualisiertes Projekt",
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

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        delete_assignment_core(&client, &href, &base_url, &username, &password)
            .await
            .expect("delete_assignment_core should succeed");
    }
}
