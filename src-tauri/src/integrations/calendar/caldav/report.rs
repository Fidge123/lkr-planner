use chrono::NaiveDate;
use tauri_plugin_http::reqwest::Method;

use super::super::ical::parse_ical_events;
use super::super::types::RawVEvent;
use super::write::CaldavSession;

pub(crate) async fn fetch_calendar_events(
    session: &CaldavSession,
    calendar_url: &str,
    week_start: NaiveDate,
) -> Result<Vec<RawVEvent>, String> {
    let week_end = week_start + chrono::Duration::days(7);
    fetch_events_in_range(session, calendar_url, week_start, week_end).await
}

pub(super) async fn fetch_events_in_range(
    session: &CaldavSession,
    calendar_url: &str,
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> Result<Vec<RawVEvent>, String> {
    let start_str = range_start.format("%Y%m%dT000000Z").to_string();
    let end_str = range_end.format("%Y%m%dT000000Z").to_string();

    let body = build_report_body(&start_str, &end_str);

    let response = session
        .client
        .request(
            Method::from_bytes(b"REPORT").expect("REPORT is a valid HTTP method"),
            calendar_url,
        )
        .basic_auth(&session.username, Some(&session.password))
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

fn parse_caldav_report(xml_text: &str) -> Result<Vec<RawVEvent>, String> {
    let doc = roxmltree::Document::parse(xml_text)
        .map_err(|e| format!("XML konnte nicht geparst werden: {e}"))?;

    let mut events = Vec::new();
    for node in doc.descendants() {
        let is_caldav = node.has_tag_name(("urn:ietf:params:xml:ns:caldav", "calendar-data"));
        let is_bare = !is_caldav && node.tag_name().name() == "calendar-data";
        if is_caldav || is_bare {
            if let Some(text) = node.text() {
                let response_node = node
                    .ancestors()
                    .find(|a| a.has_tag_name(("DAV:", "response")));
                let href = response_node
                    .and_then(|response| {
                        response
                            .children()
                            .find(|c| c.has_tag_name(("DAV:", "href")))
                            .and_then(|h| h.text())
                    })
                    .unwrap_or("")
                    .to_string();
                let etag = response_node
                    .and_then(|response| {
                        response
                            .descendants()
                            .find(|d| {
                                d.has_tag_name(("DAV:", "getetag"))
                                    || d.tag_name().name() == "getetag"
                            })
                            .and_then(|e| e.text())
                    })
                    .unwrap_or("")
                    .to_string();

                let mut parsed = parse_ical_events(text)?;
                for event in &mut parsed {
                    event.href = href.clone();
                    event.etag = etag.clone();
                    event.raw_ical = text.to_string();
                }
                events.extend(parsed);
            }
        }
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_caldav_report_captures_etag_and_raw_ical_per_event() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/calendars/user/calendar/event1.ics</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"etag-123"</d:getetag>
        <c:calendar-data>BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test-uid-1
SUMMARY:Projekt Nord
LOCATION:Baustelle Nord
DTSTART:20260505T080000
DTEND:20260505T160000
END:VEVENT
END:VCALENDAR
</c:calendar-data>
      </d:prop>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

        let events = parse_caldav_report(xml).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].etag, "\"etag-123\"");
        assert!(
            events[0].raw_ical.contains("LOCATION:Baustelle Nord"),
            "raw_ical must keep properties the parser does not model"
        );
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
}
