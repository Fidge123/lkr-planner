use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};

use super::types::RawVEvent;

/// Builds an RFC 5545 VCALENDAR payload for a lkr-planner assignment.
/// Uses a fixed floating 08:00–16:00 time window (local time, no timezone).
pub(crate) fn build_ical_payload(
    uid: &str,
    date: &str,
    summary: &str,
    project_ref: &str,
) -> String {
    let compact = date.replace('-', "");
    let dtstart = format!("{compact}T080000");
    let dtend = format!("{compact}T160000");
    let dtstamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let summary = escape_ical_text(summary);
    let description = escape_ical_text(&format!("daylite:{project_ref}"));
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//lkr-planner//EN\r\nBEGIN:VEVENT\r\nUID:{uid}\r\nDTSTAMP:{dtstamp}\r\nDTSTART:{dtstart}\r\nDTEND:{dtend}\r\nSUMMARY:{summary}\r\nDESCRIPTION:{description}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

/// Escapes a string for use as an RFC 5545 TEXT value (e.g. SUMMARY, DESCRIPTION).
/// Per RFC 5545 §3.3.11, backslash, semicolon, comma, and newlines must be escaped.
/// Backslash is escaped first so the escape characters added afterwards are not doubled.
/// Line folding (lines > 75 octets) is not implemented; CalDAV servers accept unfolded
/// lines and assignment summaries are short in practice.
fn escape_ical_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace(['\n', '\r'], "\\n")
}

/// Parses iCal text and returns all VEVENT entries found, or an error if the text is not
/// valid iCal. Uses the `icalendar` crate for RFC 5545-compliant parsing (line unfolding,
/// text unescaping, typed DTSTART). `RawVEvent.dtstart` is already in `yyyy-MM-dd` format.
pub(super) fn parse_ical_events(ical_text: &str) -> Result<Vec<RawVEvent>, String> {
    let calendar: Calendar = ical_text
        .parse()
        .map_err(|e| format!("iCal-Daten konnten nicht gelesen werden: {e:?}"))?;

    let events = calendar
        .components
        .into_iter()
        .filter_map(|component| {
            let CalendarComponent::Event(event) = component else {
                return None;
            };

            let start = event.get_start()?;
            let date = match &start {
                DatePerhapsTime::Date(d) => d.format("%Y-%m-%d").to_string(),
                DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt)) => {
                    dt.date().format("%Y-%m-%d").to_string()
                }
                DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt)) => {
                    dt.date_naive().format("%Y-%m-%d").to_string()
                }
                DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone { date_time, .. }) => {
                    date_time.date().format("%Y-%m-%d").to_string()
                }
            };
            let start_time = ical_time(&start);
            let raw_end = event.get_end();
            let end_time = raw_end.as_ref().and_then(ical_time);
            // Only DATE-valued DTEND is captured; DATE-TIME DTEND is intentionally ignored here
            // because absence expansion only applies to all-day (VALUE=DATE) events.
            let dtend = raw_end.as_ref().and_then(|dt| match dt {
                DatePerhapsTime::Date(d) => Some(*d),
                _ => None,
            });

            Some(RawVEvent {
                uid: event.get_uid().unwrap_or("").to_string(),
                summary: event.get_summary().unwrap_or("").to_string(),
                description: event.get_description().unwrap_or("").to_string(),
                dtstart: date,
                dtend,
                start_time,
                end_time,
                href: String::new(), // populated by parse_caldav_report from d:href
            })
        })
        .collect();

    Ok(events)
}

/// Extracts the time component from a `DatePerhapsTime` as an `HH:MM` string.
/// Returns `None` for all-day (date-only) values.
fn ical_time(dt: &DatePerhapsTime) -> Option<String> {
    match dt {
        DatePerhapsTime::Date(_) => None,
        DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt)) => {
            Some(dt.time().format("%H:%M").to_string())
        }
        DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt)) => {
            Some(dt.time().format("%H:%M").to_string())
        }
        DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone { date_time, .. }) => {
            Some(date_time.time().format("%H:%M").to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn parses_vevent_with_all_properties() {
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:test-uid-1\r\nSUMMARY:Projekt Nord\r\nDESCRIPTION:daylite:/v1/projects/3001\r\nDTSTART;VALUE=DATE:20260126\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let events = parse_ical_events(ical).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].uid, "test-uid-1");
        assert_eq!(events[0].summary, "Projekt Nord");
        assert_eq!(events[0].description, "daylite:/v1/projects/3001");
        assert_eq!(events[0].dtstart, "2026-01-26");
    }

    #[test]
    fn parses_multiple_vevents_from_single_ical_text() {
        let ical = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:ev-1\nSUMMARY:A\nDTSTART:20260126T080000\nEND:VEVENT\nBEGIN:VEVENT\nUID:ev-2\nSUMMARY:B\nDTSTART:20260127T080000\nEND:VEVENT\nEND:VCALENDAR\n";

        let events = parse_ical_events(ical).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "ev-1");
        assert_eq!(events[1].uid, "ev-2");
    }

    #[test]
    fn skips_vevent_without_parseable_dtstart() {
        let ical = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:bad\nSUMMARY:Bad\nDTSTART:not-a-date\nEND:VEVENT\nEND:VCALENDAR\n";

        let events = parse_ical_events(ical).unwrap();

        assert!(events.is_empty());
    }

    #[test]
    fn parse_ical_events_captures_dtend_for_all_day_event() {
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:abs-1\r\nSUMMARY:Urlaub\r\nDTSTART;VALUE=DATE:20260427\r\nDTEND;VALUE=DATE:20260502\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let events = parse_ical_events(ical).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].dtend,
            Some(NaiveDate::from_ymd_opt(2026, 5, 2).unwrap())
        );
    }

    #[test]
    fn malformed_ical_text_returns_error() {
        let result = parse_ical_events("this is definitely not valid ical");
        assert!(result.is_err(), "expected Err for malformed iCal, got Ok");
    }

    #[test]
    fn build_ical_payload_contains_expected_fields() {
        let payload = build_ical_payload(
            "test-uid-1",
            "2026-05-06",
            "Mein Projekt",
            "/v1/projects/42",
        );

        assert!(payload.contains("BEGIN:VCALENDAR"), "missing VCALENDAR");
        assert!(payload.contains("BEGIN:VEVENT"), "missing VEVENT");
        assert!(payload.contains("UID:test-uid-1"), "missing UID");
        assert!(payload.contains("DTSTART:20260506T080000"), "wrong DTSTART");
        assert!(payload.contains("DTEND:20260506T160000"), "wrong DTEND");
        assert!(payload.contains("SUMMARY:Mein Projekt"), "missing SUMMARY");
        assert!(
            payload.contains("DESCRIPTION:daylite:/v1/projects/42"),
            "missing DESCRIPTION"
        );
        assert!(payload.contains("END:VEVENT"), "missing END:VEVENT");
        assert!(payload.contains("END:VCALENDAR"), "missing END:VCALENDAR");
    }

    #[test]
    fn build_ical_payload_uses_floating_local_time_no_z_suffix() {
        let payload = build_ical_payload("uid-2", "2026-12-31", "Test", "/v1/projects/1");
        assert!(
            payload.contains("DTSTART:20261231T080000\r\n"),
            "DTSTART must not have Z suffix"
        );
        assert!(
            payload.contains("DTEND:20261231T160000\r\n"),
            "DTEND must not have Z suffix"
        );
    }

    #[test]
    fn build_ical_payload_escapes_special_chars_in_summary() {
        let payload = build_ical_payload(
            "uid-esc",
            "2026-05-06",
            "Müller, Söhne; Bau \\ Test",
            "/v1/projects/42",
        );
        assert!(
            payload.contains("SUMMARY:Müller\\, Söhne\\; Bau \\\\ Test"),
            "comma, semicolon and backslash must be escaped, got: {payload}"
        );
    }

    #[test]
    fn build_ical_payload_escapes_newline_in_summary_to_literal() {
        let payload =
            build_ical_payload("uid-nl", "2026-05-06", "Zeile1\nZeile2", "/v1/projects/42");
        assert!(
            payload.contains("SUMMARY:Zeile1\\nZeile2"),
            "newline must become the two-char escape, got: {payload}"
        );
        assert!(
            !payload.contains("SUMMARY:Zeile1\r\nZeile2"),
            "a raw newline must not split the SUMMARY property line"
        );
    }

    #[test]
    fn build_ical_payload_keeps_path_separators_in_description() {
        // Forward slashes are not RFC 5545 special characters and must survive so the
        // daylite: project reference round-trips through classification on read-back.
        let payload = build_ical_payload("uid-d", "2026-05-06", "Projekt", "/v1/projects/42");
        assert!(
            payload.contains("DESCRIPTION:daylite:/v1/projects/42"),
            "got: {payload}"
        );
    }
}
