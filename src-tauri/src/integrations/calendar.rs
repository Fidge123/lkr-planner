use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::time::Duration;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::Method;

use crate::integrations::local_store::DayliteCache;

const DAYLITE_DESCRIPTION_PREFIX: &str = "daylite:";

// ── Public types (exposed to frontend via Tauri Specta) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventKind {
    /// A lkr-planner assignment linked to a Daylite project via DESCRIPTION.
    Assignment,
    /// A bare calendar event with no Daylite project link (legacy, blocker, absence).
    Bare,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CalendarCellEvent {
    pub uid: String,
    pub kind: CalendarEventKind,
    pub title: String,
    /// Daylite project status string if resolved (e.g. "in_progress"). None for bare or unresolved.
    pub project_status: Option<String>,
    /// ISO date in the form yyyy-MM-dd.
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeWeekEvents {
    pub employee_reference: String,
    pub events: Vec<CalendarCellEvent>,
    /// Set when the CalDAV fetch for this employee fails entirely.
    pub error: Option<String>,
}

// ── Internal types ────────────────────────────────────────────────────────────

/// A raw VEVENT as parsed from iCal text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RawVEvent {
    uid: String,
    summary: String,
    description: String,
    dtstart: String,
}

/// After initial classification: either a lkr-planner event or a bare event, pending project resolution.
struct PendingEvent {
    uid: String,
    date: String,
    summary: String,
    /// None = bare event. Some(ref) = lkr-planner event with unresolved Daylite project ref.
    project_ref: Option<String>,
}

// ── Tauri command ─────────────────────────────────────────────────────────────

/// Loads all calendar events for every configured employee for the given week.
/// Returns one entry per employee that has a primary calendar configured.
/// Per-employee CalDAV failures are returned inline in `error`; only total failures
/// (store unavailable, bad date) return an `Err`.
#[tauri::command]
#[specta::specta]
pub async fn load_week_events(
    app: tauri::AppHandle,
    week_start: String,
) -> Result<Vec<EmployeeWeekEvents>, String> {
    let store = crate::integrations::local_store::load_local_store(app.clone())
        .map_err(|e| e.user_message)?;

    let week_start_date = NaiveDate::parse_from_str(&week_start, "%Y-%m-%d")
        .map_err(|_| format!("Ungültiges Wochenstartdatum: {week_start}"))?;

    let (username, password) = match crate::integrations::zep::load_zep_credentials_from_keychain()
    {
        Ok(c) => (c.username, c.password),
        Err(e) => {
            // No credentials: return error for every employee with a calendar URL
            let results = store
                .employee_settings
                .iter()
                .filter(|s| {
                    s.zep_primary_calendar
                        .as_deref()
                        .map(|u| !u.is_empty())
                        .unwrap_or(false)
                })
                .map(|s| EmployeeWeekEvents {
                    employee_reference: s.daylite_contact_reference.clone(),
                    events: vec![],
                    error: Some(e.user_message.clone()),
                })
                .collect();
            return Ok(results);
        }
    };

    // First pass: fetch CalDAV events per employee and classify against the local cache.
    let mut pending_per_employee: Vec<(String, Vec<PendingEvent>)> = Vec::new();
    let mut error_results: Vec<EmployeeWeekEvents> = Vec::new();

    for setting in &store.employee_settings {
        let employee_ref = setting.daylite_contact_reference.clone();
        let calendar_url = match setting
            .zep_primary_calendar
            .as_deref()
            .filter(|u| !u.is_empty())
        {
            Some(url) => url.to_string(),
            None => continue,
        };

        match fetch_calendar_events(&calendar_url, &username, &password, week_start_date).await {
            Ok(raw_events) => {
                let pending = raw_events.into_iter().map(classify_event).collect();
                pending_per_employee.push((employee_ref, pending));
            }
            Err(error_msg) => {
                error_results.push(EmployeeWeekEvents {
                    employee_reference: employee_ref,
                    events: vec![],
                    error: Some(error_msg),
                });
            }
        }
    }

    // Collect unique project refs that are not in the local Daylite cache.
    let mut missing_refs: Vec<String> = Vec::new();
    for (_, pending_events) in &pending_per_employee {
        for event in pending_events {
            if let Some(ref project_ref) = event.project_ref {
                let in_cache = store
                    .daylite_cache
                    .projects
                    .iter()
                    .any(|p| p.reference == *project_ref);
                if !in_cache && !missing_refs.contains(project_ref) {
                    missing_refs.push(project_ref.clone());
                }
            }
        }
    }

    // Second pass: fetch missing projects from the Daylite API (sequential, typically few).
    let mut api_results: HashMap<String, Option<(String, String)>> = HashMap::new();
    for project_ref in missing_refs {
        let result = crate::integrations::daylite::projects::fetch_project_by_reference(
            app.clone(),
            &project_ref,
        )
        .await;
        api_results.insert(project_ref, result);
    }

    // Build final results combining cache and API lookups.
    let mut results = error_results;
    for (employee_ref, pending_events) in pending_per_employee {
        let events = pending_events
            .into_iter()
            .map(|p| resolve_event(p, &store.daylite_cache, &api_results))
            .collect();
        results.push(EmployeeWeekEvents {
            employee_reference: employee_ref,
            events,
            error: None,
        });
    }

    Ok(results)
}

// ── CalDAV fetch ──────────────────────────────────────────────────────────────

async fn fetch_calendar_events(
    calendar_url: &str,
    username: &str,
    password: &str,
    week_start: NaiveDate,
) -> Result<Vec<RawVEvent>, String> {
    let week_end = week_start + chrono::Duration::days(7);
    let start_str = week_start.format("%Y%m%dT000000Z").to_string();
    let end_str = week_end.format("%Y%m%dT000000Z").to_string();

    let body = build_report_body(&start_str, &end_str);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

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

// ── iCal parsing ──────────────────────────────────────────────────────────────

/// Parses a CalDAV REPORT XML response and extracts VEVENT entries from each calendar-data element.
fn parse_caldav_report(xml_text: &str) -> Result<Vec<RawVEvent>, String> {
    let doc = roxmltree::Document::parse(xml_text)
        .map_err(|e| format!("XML konnte nicht geparst werden: {e}"))?;

    let mut events = Vec::new();
    for node in doc.descendants() {
        let is_caldav = node.has_tag_name(("urn:ietf:params:xml:ns:caldav", "calendar-data"));
        let is_bare = !is_caldav && node.tag_name().name() == "calendar-data";
        if is_caldav || is_bare {
            if let Some(text) = node.text() {
                events.extend(parse_ical_events(text));
            }
        }
    }

    Ok(events)
}

/// Parses iCal text and returns all VEVENT entries found.
pub(crate) fn parse_ical_events(ical_text: &str) -> Vec<RawVEvent> {
    let unfolded = unfold_ical_lines(ical_text);
    let mut events = Vec::new();
    let mut in_vevent = false;
    let mut current = RawVEvent::default();

    for line in unfolded.lines() {
        if line == "BEGIN:VEVENT" {
            in_vevent = true;
            current = RawVEvent::default();
        } else if line == "END:VEVENT" && in_vevent {
            in_vevent = false;
            // Only keep events with a parseable date
            if parse_ical_date(&current.dtstart).is_some() {
                events.push(current);
            }
            current = RawVEvent::default();
        } else if in_vevent {
            if let Some((name, value)) = parse_property_line(line) {
                match name.as_str() {
                    "UID" => current.uid = value,
                    "SUMMARY" => current.summary = value,
                    "DESCRIPTION" => current.description = value,
                    "DTSTART" => current.dtstart = value,
                    _ => {}
                }
            }
        }
    }

    events
}

/// Removes iCal line folding: CRLF or LF followed by a single space or tab is a continuation.
fn unfold_ical_lines(text: &str) -> String {
    text.replace("\r\n ", "")
        .replace("\r\n\t", "")
        .replace("\n ", "")
        .replace("\n\t", "")
}

/// Parses an iCal property line into `(name, value)`.
/// Property parameters (e.g. `DTSTART;VALUE=DATE:20260126`) are stripped from the name.
fn parse_property_line(line: &str) -> Option<(String, String)> {
    let colon_pos = line.find(':')?;
    let name_part = &line[..colon_pos];
    let value = line[colon_pos + 1..].to_string();
    let name = name_part.split(';').next()?.to_uppercase();
    Some((name, value))
}

/// Extracts an ISO date (yyyy-MM-dd) from an iCal DTSTART value.
/// Handles both `VALUE=DATE` format (`20260126`) and datetime format (`20260126T080000[Z]`).
pub(crate) fn parse_ical_date(dtstart: &str) -> Option<String> {
    let date_part = dtstart.split('T').next()?;
    if date_part.len() == 8 && date_part.chars().all(|c| c.is_ascii_digit()) {
        Some(format!(
            "{}-{}-{}",
            &date_part[0..4],
            &date_part[4..6],
            &date_part[6..8]
        ))
    } else {
        None
    }
}

// ── Event classification ──────────────────────────────────────────────────────

/// Classifies a raw VEVENT as a lkr-planner assignment or a bare calendar event.
fn classify_event(event: RawVEvent) -> PendingEvent {
    let date = parse_ical_date(&event.dtstart).unwrap_or_default();

    let uid = if event.uid.is_empty() {
        // Synthesise a stable-ish UID from the event content when none is provided.
        format!("synthetic-{}-{}", date, event.summary.len())
    } else {
        event.uid
    };

    // iCal encodes literal newlines in property values as backslash-n.
    let description_unescaped = event.description.replace("\\n", "\n").replace("\\N", "\n");

    let first_line = description_unescaped.lines().next().unwrap_or("").trim();

    let project_ref = if first_line.starts_with(DAYLITE_DESCRIPTION_PREFIX) {
        let raw_ref = first_line[DAYLITE_DESCRIPTION_PREFIX.len()..].trim();
        if raw_ref.is_empty() {
            None
        } else {
            Some(raw_ref.to_string())
        }
    } else {
        None
    };

    PendingEvent {
        uid,
        date,
        summary: event.summary,
        project_ref,
    }
}

// ── Project resolution ────────────────────────────────────────────────────────

/// Resolves a pending event into a `CalendarCellEvent` using the Daylite cache and
/// pre-fetched API results. Falls back to a German placeholder if the project cannot
/// be resolved.
fn resolve_event(
    pending: PendingEvent,
    cache: &DayliteCache,
    api_results: &HashMap<String, Option<(String, String)>>,
) -> CalendarCellEvent {
    let Some(project_ref) = pending.project_ref else {
        return CalendarCellEvent {
            uid: pending.uid,
            kind: CalendarEventKind::Bare,
            title: pending.summary,
            project_status: None,
            date: pending.date,
        };
    };

    // Try the local Daylite cache first.
    if let Some(cached) = cache.projects.iter().find(|p| p.reference == project_ref) {
        return CalendarCellEvent {
            uid: pending.uid,
            kind: CalendarEventKind::Assignment,
            title: cached.name.clone(),
            project_status: Some(cached.status.clone()),
            date: pending.date,
        };
    }

    // Try the pre-fetched API result.
    if let Some(Some((name, status))) = api_results.get(&project_ref) {
        return CalendarCellEvent {
            uid: pending.uid,
            kind: CalendarEventKind::Assignment,
            title: name.clone(),
            project_status: Some(status.clone()),
            date: pending.date,
        };
    }

    // Placeholder: project could not be resolved.
    CalendarCellEvent {
        uid: pending.uid,
        kind: CalendarEventKind::Assignment,
        title: format!(
            "Beschreibung für {} konnte nicht abgerufen werden",
            pending.summary
        ),
        project_status: None,
        date: pending.date,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::local_store::DayliteProjectCacheEntry;

    // ── iCal parsing ──

    #[test]
    fn parses_vevent_with_all_properties() {
        let ical = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:test-uid-1\r\nSUMMARY:Projekt Nord\r\nDESCRIPTION:daylite:/v1/projects/3001\r\nDTSTART;VALUE=DATE:20260126\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let events = parse_ical_events(ical);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].uid, "test-uid-1");
        assert_eq!(events[0].summary, "Projekt Nord");
        assert_eq!(events[0].description, "daylite:/v1/projects/3001");
        assert_eq!(events[0].dtstart, "20260126");
    }

    #[test]
    fn parses_multiple_vevents_from_single_ical_text() {
        let ical = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:ev-1\nSUMMARY:A\nDTSTART:20260126T080000\nEND:VEVENT\nBEGIN:VEVENT\nUID:ev-2\nSUMMARY:B\nDTSTART:20260127T080000\nEND:VEVENT\nEND:VCALENDAR\n";

        let events = parse_ical_events(ical);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "ev-1");
        assert_eq!(events[1].uid, "ev-2");
    }

    #[test]
    fn skips_vevent_without_parseable_dtstart() {
        let ical = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:bad\nSUMMARY:Bad\nDTSTART:not-a-date\nEND:VEVENT\nEND:VCALENDAR\n";

        let events = parse_ical_events(ical);

        assert!(events.is_empty());
    }

    #[test]
    fn unfolds_crlf_folded_lines() {
        let folded = "DESCRIPTION:This is a very long description that has been\r\n folded here\r\n and here";
        let expected =
            "DESCRIPTION:This is a very long description that has been folded hereand here";

        assert_eq!(unfold_ical_lines(folded), expected);
    }

    #[test]
    fn unfolds_lf_folded_lines() {
        let folded = "SUMMARY:Folded\n line";
        let expected = "SUMMARY:Foldedline";

        assert_eq!(unfold_ical_lines(folded), expected);
    }

    #[test]
    fn parses_date_only_dtstart() {
        assert_eq!(parse_ical_date("20260126"), Some("2026-01-26".to_string()));
    }

    #[test]
    fn parses_datetime_dtstart() {
        assert_eq!(
            parse_ical_date("20260126T080000"),
            Some("2026-01-26".to_string())
        );
    }

    #[test]
    fn parses_datetime_utc_dtstart() {
        assert_eq!(
            parse_ical_date("20260126T080000Z"),
            Some("2026-01-26".to_string())
        );
    }

    #[test]
    fn returns_none_for_invalid_dtstart() {
        assert_eq!(parse_ical_date("not-a-date"), None);
        assert_eq!(parse_ical_date(""), None);
    }

    // ── Event classification ──

    #[test]
    fn classifies_lkr_planner_event_with_daylite_description() {
        let event = RawVEvent {
            uid: "uid-1".to_string(),
            summary: "Projekt Nord".to_string(),
            description: "daylite:/v1/projects/3001".to_string(),
            dtstart: "20260126".to_string(),
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, Some("/v1/projects/3001".to_string()));
        assert_eq!(pending.date, "2026-01-26");
        assert_eq!(pending.summary, "Projekt Nord");
    }

    #[test]
    fn classifies_bare_event_without_daylite_description() {
        let event = RawVEvent {
            uid: "uid-2".to_string(),
            summary: "Auto Werkstatt".to_string(),
            description: "Bitte Auto abholen".to_string(),
            dtstart: "20260127".to_string(),
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, None);
        assert_eq!(pending.summary, "Auto Werkstatt");
    }

    #[test]
    fn classifies_bare_event_with_empty_description() {
        let event = RawVEvent {
            uid: "uid-3".to_string(),
            summary: "Blockertermin".to_string(),
            description: String::new(),
            dtstart: "20260128".to_string(),
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, None);
    }

    #[test]
    fn classifies_event_with_multiline_description_using_first_line_only() {
        let event = RawVEvent {
            uid: "uid-4".to_string(),
            summary: "Projekt Süd".to_string(),
            description: "daylite:/v1/projects/4001\\nZusätzliche Notizen hier".to_string(),
            dtstart: "20260129".to_string(),
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, Some("/v1/projects/4001".to_string()));
    }

    #[test]
    fn synthesises_uid_for_event_without_uid() {
        let event = RawVEvent {
            uid: String::new(),
            summary: "Ohne UID".to_string(),
            description: String::new(),
            dtstart: "20260126".to_string(),
        };

        let pending = classify_event(event);

        assert!(!pending.uid.is_empty());
        assert!(pending.uid.starts_with("synthetic-"));
    }

    // ── Project resolution ──

    #[test]
    fn resolves_assignment_event_from_cache() {
        let pending = PendingEvent {
            uid: "uid-1".to_string(),
            date: "2026-01-26".to_string(),
            summary: "Projekt Nord".to_string(),
            project_ref: Some("/v1/projects/3001".to_string()),
        };
        let cache = DayliteCache {
            last_synced_at: None,
            projects: vec![DayliteProjectCacheEntry {
                reference: "/v1/projects/3001".to_string(),
                name: "Projekt Nord".to_string(),
                status: "in_progress".to_string(),
            }],
            contacts: vec![],
        };
        let api_results = HashMap::new();

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Nord");
        assert_eq!(event.project_status, Some("in_progress".to_string()));
        assert_eq!(event.date, "2026-01-26");
    }

    #[test]
    fn resolves_assignment_event_from_api_result() {
        let pending = PendingEvent {
            uid: "uid-2".to_string(),
            date: "2026-01-27".to_string(),
            summary: "Projekt Süd".to_string(),
            project_ref: Some("/v1/projects/4001".to_string()),
        };
        let cache = DayliteCache::default();
        let mut api_results = HashMap::new();
        api_results.insert(
            "/v1/projects/4001".to_string(),
            Some(("Projekt Süd".to_string(), "deferred".to_string())),
        );

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Süd");
        assert_eq!(event.project_status, Some("deferred".to_string()));
    }

    #[test]
    fn shows_placeholder_when_project_not_resolvable() {
        let pending = PendingEvent {
            uid: "uid-3".to_string(),
            date: "2026-01-28".to_string(),
            summary: "Unbekanntes Projekt".to_string(),
            project_ref: Some("/v1/projects/9999".to_string()),
        };
        let cache = DayliteCache::default();
        let mut api_results = HashMap::new();
        api_results.insert("/v1/projects/9999".to_string(), None);

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert!(event
            .title
            .contains("Beschreibung für Unbekanntes Projekt konnte nicht abgerufen werden"));
        assert_eq!(event.project_status, None);
    }

    #[test]
    fn resolves_bare_event() {
        let pending = PendingEvent {
            uid: "uid-4".to_string(),
            date: "2026-01-29".to_string(),
            summary: "Auto Werkstatt".to_string(),
            project_ref: None,
        };
        let cache = DayliteCache::default();
        let api_results = HashMap::new();

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Bare);
        assert_eq!(event.title, "Auto Werkstatt");
        assert_eq!(event.project_status, None);
    }
}
