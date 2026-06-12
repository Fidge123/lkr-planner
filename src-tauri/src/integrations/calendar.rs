use chrono::NaiveDate;
use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::Method;
use uuid::Uuid;

use crate::integrations::local_store::DayliteCache;

const DAYLITE_DESCRIPTION_PREFIX: &str = "daylite:";

// ── Public types (exposed to frontend via Tauri Specta) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventKind {
    /// A lkr-planner assignment linked to a Daylite project via DESCRIPTION.
    Assignment,
    /// A bare calendar event with no Daylite project link (legacy, blocker, appointment).
    Bare,
    /// An all-day absence from the employee's dedicated ZEP absence calendar.
    Absence,
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
    /// Start time in HH:MM format. None for all-day events.
    pub start_time: Option<String>,
    /// End time in HH:MM format. None for all-day events.
    pub end_time: Option<String>,
    /// CalDAV resource URL (d:href from REPORT) needed for PUT/DELETE. None if unknown.
    pub href: Option<String>,
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
/// `dtstart` holds an ISO date string in the form `yyyy-MM-dd` (already formatted).
/// `dtend` is populated only for all-day events (DATE value); timed events use `end_time`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RawVEvent {
    uid: String,
    summary: String,
    description: String,
    dtstart: String,
    /// Exclusive end date for all-day events (DATE values only).
    /// RFC 5545 §3.8.2.2: for DATE-only values, DTEND is the day after the last covered day (exclusive).
    /// DATE-TIME DTEND is intentionally not stored here; timed events use `end_time` instead.
    dtend: Option<NaiveDate>,
    start_time: Option<String>,
    end_time: Option<String>,
    /// CalDAV resource URL from d:href in REPORT response. Empty if not found.
    href: String,
}

/// After initial classification: either a lkr-planner event or a bare event, pending project resolution.
struct PendingEvent {
    uid: String,
    date: String,
    summary: String,
    /// None = bare event. Some(ref) = lkr-planner event with unresolved Daylite project ref.
    project_ref: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    /// CalDAV resource URL (d:href) required for PUT/DELETE operations. Empty if unknown.
    href: String,
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
            let results: Vec<EmployeeWeekEvents> = store
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
            if results.is_empty() {
                eprintln!(
                    "load_week_events: ZEP credentials unavailable and no primary calendars configured: {}",
                    e.technical_message
                );
            }
            return Ok(results);
        }
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    // First pass: fetch CalDAV events for all employees concurrently (primary + absence per
    // employee are also concurrent via tokio::join!).
    let employee_futures: Vec<_> = store
        .employee_settings
        .iter()
        .filter_map(|setting| {
            let calendar_url = setting
                .zep_primary_calendar
                .as_deref()
                .filter(|u| !u.is_empty())
                .map(str::to_string)?;

            let absence_url = setting
                .zep_absence_calendar
                .as_deref()
                .filter(|u| !u.is_empty())
                .map(str::to_string);

            let employee_ref = setting.daylite_contact_reference.clone();
            let client = client.clone();
            let username = username.clone();
            let password = password.clone();

            Some(async move {
                let (primary_result, absence_result) = tokio::join!(
                    fetch_calendar_events(
                        &client,
                        &calendar_url,
                        &username,
                        &password,
                        week_start_date
                    ),
                    async {
                        match absence_url {
                            Some(ref url) => fetch_calendar_events(
                                &client,
                                url,
                                &username,
                                &password,
                                week_start_date,
                            )
                            .await
                            .ok(),
                            None => None,
                        }
                    }
                );
                (employee_ref, primary_result, absence_result)
            })
        })
        .collect();

    let fetch_results = futures::future::join_all(employee_futures).await;

    let mut pending_per_employee: Vec<(String, Vec<PendingEvent>, Vec<CalendarCellEvent>)> =
        Vec::new();
    let mut error_results: Vec<EmployeeWeekEvents> = Vec::new();

    for (employee_ref, primary_result, absence_result) in fetch_results {
        match primary_result {
            Ok(raw_events) => {
                let pending = raw_events.into_iter().map(classify_event).collect();
                let absence_events = absence_result
                    .map(|raw| map_absence_raw_events_for_week(raw, week_start_date))
                    .unwrap_or_default();
                pending_per_employee.push((employee_ref, pending, absence_events));
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
    let mut missing_refs: HashSet<String> = HashSet::new();
    for (_, pending_events, _) in &pending_per_employee {
        for event in pending_events {
            if let Some(ref project_ref) = event.project_ref {
                let in_cache = store
                    .daylite_cache
                    .projects
                    .iter()
                    .any(|p| p.reference == *project_ref);
                if !in_cache {
                    missing_refs.insert(project_ref.clone());
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
    for (employee_ref, pending_events, absence_events) in pending_per_employee {
        let mut events: Vec<CalendarCellEvent> = pending_events
            .into_iter()
            .map(|p| resolve_event(p, &store.daylite_cache, &api_results))
            .collect();
        events.extend(absence_events);
        // Deduplicate by UID to guard against CalDAV servers redelivering the same event.
        let mut seen_uids = HashSet::new();
        events.retain(|e| seen_uids.insert(e.uid.clone()));
        // Absence events are shown first within each day.
        sort_events_absences_first(&mut events);
        results.push(EmployeeWeekEvents {
            employee_reference: employee_ref,
            events,
            error: None,
        });
    }

    Ok(results)
}

// ── CalDAV write commands ─────────────────────────────────────────────────────

/// Creates a new assignment event on the employee's primary CalDAV calendar.
/// Returns the CalDAV resource href (e.g. `{calendar_url}/{uid}.ics`) of the new event.
#[tauri::command]
#[specta::specta]
pub async fn create_assignment(
    app: tauri::AppHandle,
    employee_reference: String,
    date: String,
    project_ref: String,
    project_name: String,
) -> Result<String, String> {
    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let calendar_url = store
        .employee_settings
        .iter()
        .find(|s| s.daylite_contact_reference == employee_reference)
        .and_then(|s| s.zep_primary_calendar.as_deref())
        .filter(|u| !u.is_empty())
        .ok_or_else(|| "Kein Kalender für diesen Mitarbeiter konfiguriert.".to_string())?
        .to_string();

    let (username, password) = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map(|c| (c.username, c.password))
        .map_err(|e| e.user_message)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    create_assignment_core(
        &client,
        &calendar_url,
        &username,
        &password,
        &date,
        &project_ref,
        &project_name,
    )
    .await
}

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

/// Updates an existing assignment event in place using the stored CalDAV href.
#[tauri::command]
#[specta::specta]
pub async fn update_assignment(
    app: tauri::AppHandle,
    href: String,
    uid: String,
    date: String,
    project_ref: String,
    project_name: String,
) -> Result<(), String> {
    let (username, password) = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map(|c| (c.username, c.password))
        .map_err(|e| e.user_message)?;

    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let base_url = store.api_endpoints.zep_caldav_root_url.clone();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    update_assignment_core(
        &client,
        &href,
        &base_url,
        &uid,
        &username,
        &password,
        &date,
        &project_ref,
        &project_name,
    )
    .await
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

/// Deletes an assignment event using the stored CalDAV href.
#[tauri::command]
#[specta::specta]
pub async fn delete_assignment(app: tauri::AppHandle, href: String) -> Result<(), String> {
    let (username, password) = crate::integrations::zep::load_zep_credentials_from_keychain()
        .map(|c| (c.username, c.password))
        .map_err(|e| e.user_message)?;

    let store =
        crate::integrations::local_store::load_local_store(app).map_err(|e| e.user_message)?;

    let base_url = store.api_endpoints.zep_caldav_root_url.clone();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP-Client konnte nicht erstellt werden: {e}"))?;

    delete_assignment_core(&client, &href, &base_url, &username, &password).await
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

// ── iCal payload builder ──────────────────────────────────────────────────────

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

// ── CalDAV fetch ──────────────────────────────────────────────────────────────

async fn fetch_calendar_events(
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

// ── iCal parsing ──────────────────────────────────────────────────────────────

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

/// Parses iCal text and returns all VEVENT entries found, or an error if the text is not
/// valid iCal. Uses the `icalendar` crate for RFC 5545-compliant parsing (line unfolding,
/// text unescaping, typed DTSTART). `RawVEvent.dtstart` is already in `yyyy-MM-dd` format.
fn parse_ical_events(ical_text: &str) -> Result<Vec<RawVEvent>, String> {
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

// ── Event classification ──────────────────────────────────────────────────────

/// Classifies a raw VEVENT as a lkr-planner assignment or a bare calendar event.
fn classify_event(event: RawVEvent) -> PendingEvent {
    let date = event.dtstart;

    let uid = if event.uid.is_empty() {
        // Synthesise a stable-ish UID from event content. Summary is sanitized to alphanumeric
        // and hyphens only, so the UID is safe to embed in keys or URLs.
        let safe: String = event
            .summary
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(50)
            .collect();
        format!("synthetic-{date}-{safe}")
    } else {
        event.uid
    };

    // Strip ASCII whitespace, BOM (U+FEFF), and zero-width space (U+200B) that some
    // calendar UIs prepend to the description field.
    let first_line = event
        .description
        .lines()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}' || c == '\u{200b}');

    let project_ref = if let Some(stripped) = first_line.strip_prefix(DAYLITE_DESCRIPTION_PREFIX) {
        let raw_ref = stripped.trim();
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
        start_time: event.start_time,
        end_time: event.end_time,
        href: event.href,
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
    let PendingEvent {
        uid,
        date,
        summary,
        project_ref,
        start_time,
        end_time,
        href,
    } = pending;

    let href = if href.is_empty() { None } else { Some(href) };

    let Some(project_ref) = project_ref else {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Bare,
            title: summary,
            project_status: None,
            date,
            start_time,
            end_time,
            href,
        };
    };

    // Try the local Daylite cache first.
    if let Some(cached) = cache.projects.iter().find(|p| p.reference == project_ref) {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Assignment,
            title: cached.name.clone(),
            project_status: Some(cached.status.clone()),
            date,
            start_time,
            end_time,
            href,
        };
    }

    // Try the pre-fetched API result.
    if let Some(Some((name, status))) = api_results.get(&project_ref) {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Assignment,
            title: name.clone(),
            project_status: Some(status.clone()),
            date,
            start_time,
            end_time,
            href,
        };
    }

    // Placeholder: project could not be resolved.
    CalendarCellEvent {
        uid,
        kind: CalendarEventKind::Assignment,
        title: format!("Beschreibung für {} konnte nicht abgerufen werden", summary),
        project_status: None,
        date,
        start_time,
        end_time,
        href,
    }
}

// ── Event ordering ────────────────────────────────────────────────────────────

/// Sorts a mixed list of calendar events so that `Absence` events always appear
/// first within each day. Within the same kind, original relative order is preserved.
fn sort_events_absences_first(events: &mut [CalendarCellEvent]) {
    events.sort_by(|a, b| {
        let kind_order = |e: &CalendarCellEvent| {
            if matches!(e.kind, CalendarEventKind::Absence) {
                0u8
            } else {
                1u8
            }
        };
        a.date.cmp(&b.date).then(kind_order(a).cmp(&kind_order(b)))
    });
}

// ── Absence event mapping ─────────────────────────────────────────────────────

/// Maps raw absence calendar events to `CalendarCellEvent`s for the given week.
/// All-day events with a `dtend` are expanded into one event per day in
/// `[dtstart, dtend)` clamped to `[week_start, week_start + 7)`.
fn map_absence_raw_events_for_week(
    raw_events: Vec<RawVEvent>,
    week_start: NaiveDate,
) -> Vec<CalendarCellEvent> {
    let week_end = week_start + chrono::Duration::days(7);
    let mut result = Vec::new();

    for raw in raw_events {
        let event_start = match NaiveDate::parse_from_str(&raw.dtstart, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let href = if raw.href.is_empty() {
            None
        } else {
            Some(raw.href.clone())
        };

        if let Some(event_end) = raw.dtend {
            // All-day multi-day event: expand into per-day events within the week.
            let clamped_start = event_start.max(week_start);
            let clamped_end = event_end.min(week_end);
            let mut day = clamped_start;
            while day < clamped_end {
                result.push(CalendarCellEvent {
                    // NaiveDate Display format is "yyyy-MM-dd" (RFC 3339 date).
                    uid: format!("{}-{}", raw.uid, day),
                    kind: CalendarEventKind::Absence,
                    title: raw.summary.clone(),
                    project_status: None,
                    date: day.format("%Y-%m-%d").to_string(),
                    start_time: None,
                    end_time: None,
                    href: href.clone(),
                });
                day += chrono::Duration::days(1);
            }
        } else {
            result.push(CalendarCellEvent {
                uid: raw.uid,
                kind: CalendarEventKind::Absence,
                title: raw.summary,
                project_status: None,
                date: raw.dtstart,
                start_time: raw.start_time,
                end_time: raw.end_time,
                href,
            });
        }
    }

    result
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

    // ── Event classification ──

    #[test]
    fn classifies_lkr_planner_event_with_daylite_description() {
        let event = RawVEvent {
            uid: "uid-1".to_string(),
            summary: "Projekt Nord".to_string(),
            description: "daylite:/v1/projects/3001".to_string(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
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
            dtstart: "2026-01-27".to_string(),
            ..Default::default()
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
            dtstart: "2026-01-28".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, None);
    }

    #[test]
    fn classifies_event_with_multiline_description_using_first_line_only() {
        let event = RawVEvent {
            uid: "uid-4".to_string(),
            summary: "Projekt Süd".to_string(),
            description: "daylite:/v1/projects/4001\nZusätzliche Notizen hier".to_string(),
            dtstart: "2026-01-29".to_string(),
            ..Default::default()
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
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
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
            start_time: None,
            end_time: None,
            href: String::new(),
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
            start_time: None,
            end_time: None,
            href: String::new(),
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
            start_time: None,
            end_time: None,
            href: String::new(),
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
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = DayliteCache::default();
        let api_results = HashMap::new();

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Bare);
        assert_eq!(event.title, "Auto Werkstatt");
        assert_eq!(event.project_status, None);
    }

    // ── Absence event mapping ──

    #[test]
    fn absence_event_has_absence_kind_title_and_no_project_status() {
        let raw = RawVEvent {
            uid: "abs-1".to_string(),
            summary: "Urlaub".to_string(),
            description: String::new(),
            dtstart: "2026-04-28".to_string(),
            ..Default::default()
        };
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(vec![raw], week_start);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[0].title, "Urlaub");
        assert_eq!(events[0].project_status, None);
    }

    #[test]
    fn maps_multiple_absence_events_from_raw() {
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();
        let raw = vec![
            RawVEvent {
                uid: "abs-1".to_string(),
                summary: "Urlaub".to_string(),
                dtstart: "2026-04-28".to_string(),
                ..Default::default()
            },
            RawVEvent {
                uid: "abs-2".to_string(),
                summary: "Krankenstand".to_string(),
                dtstart: "2026-04-29".to_string(),
                ..Default::default()
            },
        ];

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.kind == CalendarEventKind::Absence));
        assert!(events.iter().all(|e| e.project_status.is_none()));
    }

    #[test]
    fn returns_empty_when_no_absence_raw_events() {
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();
        let events = map_absence_raw_events_for_week(vec![], week_start);
        assert!(events.is_empty());
    }

    #[test]
    fn absence_fetch_failure_produces_no_absence_events() {
        // Simulates the silent-failure path: when fetch_calendar_events returns Err,
        // the caller passes an empty vec to the mapping function.
        let raw: Vec<RawVEvent> = Vec::new();
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert!(events.is_empty());
    }

    // ── Multi-day absence expansion ──

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
    fn multi_day_absence_expands_into_one_event_per_day_in_week() {
        // Mon–Fri absence (DTEND is exclusive: Sat = last day not covered).
        let raw = vec![RawVEvent {
            uid: "abs-1".to_string(),
            summary: "Urlaub".to_string(),
            dtstart: "2026-04-27".to_string(),
            dtend: Some(NaiveDate::from_ymd_opt(2026, 5, 2).unwrap()),
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 5);
        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[4].date, "2026-05-01");
        assert!(events.iter().all(|e| e.kind == CalendarEventKind::Absence));
        assert!(events.iter().all(|e| e.title == "Urlaub"));
    }

    #[test]
    fn multi_day_absence_starting_before_week_only_covers_days_in_week() {
        // Absence starts last week (Mon Apr 20), ends Wed Apr 29 (exclusive).
        // Only Mon Apr 27 and Tue Apr 28 fall in this week.
        let raw = vec![RawVEvent {
            uid: "abs-2".to_string(),
            summary: "Krankenstand".to_string(),
            dtstart: "2026-04-20".to_string(),
            dtend: Some(NaiveDate::from_ymd_opt(2026, 4, 29).unwrap()),
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }

    // H1: DATE-TIME DTSTART with no DATE DTEND → single-day event (intentional; expansion only
    // applies when the iCal source uses VALUE=DATE for DTEND, as ZEP does for all-day absences).
    #[test]
    fn absence_with_timed_dtstart_and_no_dtend_produces_single_event() {
        let raw = vec![RawVEvent {
            uid: "abs-timed".to_string(),
            summary: "Kurzurlaub".to_string(),
            dtstart: "2026-04-28".to_string(),
            dtend: None,
            start_time: Some("08:00".to_string()),
            end_time: Some("17:00".to_string()),
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].date, "2026-04-28");
        assert_eq!(events[0].kind, CalendarEventKind::Absence);
    }

    // M3 (red): malformed iCal text should return an error from parse_ical_events.
    #[test]
    fn malformed_ical_text_returns_error() {
        let result = parse_ical_events("this is definitely not valid ical");
        assert!(result.is_err(), "expected Err for malformed iCal, got Ok");
    }

    // M5 (red): BOM-prefixed description should still classify as a Daylite event.
    #[test]
    fn classifies_event_with_bom_prefixed_daylite_description() {
        let event = RawVEvent {
            uid: "uid-bom".to_string(),
            summary: "Projekt BOM".to_string(),
            description: "\u{feff}daylite:/v1/projects/5001".to_string(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, Some("/v1/projects/5001".to_string()));
    }

    // L3 (red): synthetic UID must not contain newlines, slashes, or other special characters.
    #[test]
    fn synthetic_uid_contains_only_safe_characters() {
        let event = RawVEvent {
            uid: String::new(),
            summary: "Termin\nmit/Sonderzeichen".to_string(),
            description: String::new(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert!(!pending.uid.contains('\n'), "UID must not contain newline");
        assert!(!pending.uid.contains('/'), "UID must not contain slash");
    }

    // L8: absence events with a daylite: description must NOT be classified as assignments.
    #[test]
    fn absence_events_are_never_classified_as_assignments_regardless_of_description() {
        let raw = vec![RawVEvent {
            uid: "abs-daylite".to_string(),
            summary: "Urlaub".to_string(),
            description: "daylite:/v1/projects/9999".to_string(),
            dtstart: "2026-04-28".to_string(),
            dtend: None,
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[0].project_status, None);
    }

    // Ordering: absence events must appear before other event kinds on the same day.
    #[test]
    fn absence_sorted_before_assignment_on_same_day() {
        let mut events = vec![
            CalendarCellEvent {
                uid: "assignment-1".to_string(),
                kind: CalendarEventKind::Assignment,
                title: "Projekt".to_string(),
                project_status: Some("in_progress".to_string()),
                date: "2026-04-28".to_string(),
                start_time: Some("09:00".to_string()),
                end_time: Some("17:00".to_string()),
                href: None,
            },
            CalendarCellEvent {
                uid: "absence-1".to_string(),
                kind: CalendarEventKind::Absence,
                title: "Urlaub".to_string(),
                project_status: None,
                date: "2026-04-28".to_string(),
                start_time: None,
                end_time: None,
                href: None,
            },
        ];

        sort_events_absences_first(&mut events);

        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[1].kind, CalendarEventKind::Assignment);
    }

    #[test]
    fn absence_sorted_before_bare_event_on_same_day() {
        let mut events = vec![
            CalendarCellEvent {
                uid: "bare-1".to_string(),
                kind: CalendarEventKind::Bare,
                title: "Blocker".to_string(),
                project_status: None,
                date: "2026-04-28".to_string(),
                start_time: Some("10:00".to_string()),
                end_time: None,
                href: None,
            },
            CalendarCellEvent {
                uid: "absence-1".to_string(),
                kind: CalendarEventKind::Absence,
                title: "Urlaub".to_string(),
                project_status: None,
                date: "2026-04-28".to_string(),
                start_time: None,
                end_time: None,
                href: None,
            },
        ];

        sort_events_absences_first(&mut events);

        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[1].kind, CalendarEventKind::Bare);
    }

    #[test]
    fn absence_on_different_day_does_not_reorder_other_days() {
        let mut events = vec![
            CalendarCellEvent {
                uid: "assignment-mon".to_string(),
                kind: CalendarEventKind::Assignment,
                title: "Projekt".to_string(),
                project_status: Some("in_progress".to_string()),
                date: "2026-04-27".to_string(),
                start_time: Some("09:00".to_string()),
                end_time: None,
                href: None,
            },
            CalendarCellEvent {
                uid: "absence-tue".to_string(),
                kind: CalendarEventKind::Absence,
                title: "Urlaub".to_string(),
                project_status: None,
                date: "2026-04-28".to_string(),
                start_time: None,
                end_time: None,
                href: None,
            },
        ];

        sort_events_absences_first(&mut events);

        // Monday assignment stays before Tuesday absence (different days).
        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }

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

    #[test]
    fn href_propagates_through_classify_and_resolve_to_cell_event() {
        let event = RawVEvent {
            uid: "uid-href".to_string(),
            summary: "Projekt Nord".to_string(),
            description: "daylite:/v1/projects/3001".to_string(),
            dtstart: "2026-05-05".to_string(),
            href: "/calendars/user/cal/uid-href.ics".to_string(),
            ..Default::default()
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

        let pending = classify_event(event);
        let cell_event = resolve_event(pending, &cache, &HashMap::new());

        assert_eq!(
            cell_event.href,
            Some("/calendars/user/cal/uid-href.ics".to_string())
        );
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
