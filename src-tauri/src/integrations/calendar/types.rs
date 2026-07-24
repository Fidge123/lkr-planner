use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventKind {
    Assignment,
    Bare,
    Absence,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CalendarCellEvent {
    pub uid: String,
    pub kind: CalendarEventKind,
    pub title: String,
    pub project_status: Option<String>,
    pub date: String,
    // Start time in HH:MM format. None for all-day events.
    pub start_time: Option<String>,
    // End time in HH:MM format. None for all-day events.
    pub end_time: Option<String>,
    // CalDAV resource URL (d:href from REPORT) needed for PUT/DELETE. None if unknown.
    pub href: Option<String>,
    // Daylite project reference (e.g. "/v1/projects/42") stored in DESCRIPTION. None for bare events.
    pub project_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeWeekEvents {
    pub employee_reference: String,
    pub events: Vec<CalendarCellEvent>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RawVEvent {
    pub(super) uid: String,
    pub(super) summary: String,
    pub(super) description: String,
    pub(super) dtstart: String,
    // Exclusive end date for all-day events (DATE values only).
    pub(super) dtend: Option<NaiveDate>,
    pub(super) start_time: Option<String>,
    pub(super) end_time: Option<String>,
    pub(super) href: String,
    // ETag from d:getetag in the REPORT response; sent as If-Match on re-slot PUTs. Empty if absent.
    pub(super) etag: String,
    // Full calendar-data text of the resource, used to patch slot times without dropping
    // user-added properties. Empty when the event was not parsed from a REPORT.
    pub(super) raw_ical: String,
}

pub(super) struct PendingEvent {
    pub(super) uid: String,
    pub(super) date: String,
    pub(super) summary: String,
    // None = bare event. Some(ref) = lkr-planner event with unresolved Daylite project ref.
    pub(super) project_ref: Option<String>,
    pub(super) start_time: Option<String>,
    pub(super) end_time: Option<String>,
    pub(super) href: String,
}
