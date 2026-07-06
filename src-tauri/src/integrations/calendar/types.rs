use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;

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
    /// Daylite project reference (e.g. "/v1/projects/42") stored in DESCRIPTION. None for bare events.
    pub project_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeWeekEvents {
    pub employee_reference: String,
    pub events: Vec<CalendarCellEvent>,
    /// Set when the CalDAV fetch for this employee fails entirely.
    pub error: Option<String>,
}

/// A raw VEVENT as parsed from iCal text.
/// `dtstart` holds an ISO date string in the form `yyyy-MM-dd` (already formatted).
/// `dtend` is populated only for all-day events (DATE value); timed events use `end_time`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RawVEvent {
    pub(super) uid: String,
    pub(super) summary: String,
    pub(super) description: String,
    pub(super) dtstart: String,
    /// Exclusive end date for all-day events (DATE values only).
    /// RFC 5545 §3.8.2.2: for DATE-only values, DTEND is the day after the last covered day (exclusive).
    /// DATE-TIME DTEND is intentionally not stored here; timed events use `end_time` instead.
    pub(super) dtend: Option<NaiveDate>,
    pub(super) start_time: Option<String>,
    pub(super) end_time: Option<String>,
    /// CalDAV resource URL from d:href in REPORT response. Empty if not found.
    pub(super) href: String,
}

/// After initial classification: either a lkr-planner event or a bare event, pending project resolution.
pub(super) struct PendingEvent {
    pub(super) uid: String,
    pub(super) date: String,
    pub(super) summary: String,
    /// None = bare event. Some(ref) = lkr-planner event with unresolved Daylite project ref.
    pub(super) project_ref: Option<String>,
    pub(super) start_time: Option<String>,
    pub(super) end_time: Option<String>,
    /// CalDAV resource URL (d:href) required for PUT/DELETE operations. Empty if unknown.
    pub(super) href: String,
}
