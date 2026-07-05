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

/// Outcome of moving an assignment from one employee's calendar to another.
/// CalDAV has no atomic cross-collection move, so the target copy is created first
/// and the source deleted afterwards; a failed source delete yields a partial move.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MoveAssignmentResult {
    /// Target created and source deleted.
    #[serde(rename_all = "camelCase")]
    Moved { new_href: String },
    /// Target created but the source delete failed; the assignment now exists twice.
    #[serde(rename_all = "camelCase")]
    SourceDeleteFailed {
        new_href: String,
        source_href: String,
    },
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
