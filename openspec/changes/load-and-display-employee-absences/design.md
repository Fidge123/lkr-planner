## Context

The local store already has `zep_absence_calendar`, `absence_ical_last_tested_at`, and `absence_ical_last_test_passed` on `EmployeeSetting`. The `EmployeeIcalDialog` already renders a full "Abwesenheit" section for configuring and testing the absence calendar URL. The `IcalSource` enum already has `Primary` and `Absence` variants.

What is missing: the backend does not fetch from absence calendars, `CalendarEventKind` has no `Absence` variant, and the frontend does not render absence events distinctly.

## Goals / Non-Goals

**Goals:**
- Fetch events from `zep_absence_calendar` per employee in `load_week_events`
- Surface absence events as `CalendarEventKind::Absence` in the existing response structure
- Render absence events visually distinct from assignments and bare events in `TimetableCell`
- No new Tauri command; no changes to settings UI (already complete)

**Non-Goals:**
- Showing absence error states in the timetable row (absence is optional; failures degrade silently)
- Distinguishing absence subtypes (Urlaub vs. Krankenstand)

## Decisions

### Extend `load_week_events`, not a new command
Absence events are fetched in the same command call as primary calendar events. This avoids a second round-trip when loading the week view. Absence events are appended to the same `events: Vec<CalendarCellEvent>` per employee using the new `Absence` kind.

### Add `Absence` to `CalendarEventKind`
```rust
pub enum CalendarEventKind {
    Assignment,
    Bare,
    Absence,  // new
}
```
Absence events carry a `title` (the iCal SUMMARY, e.g. "Urlaub") and `date`, but no `project_status`.

### Concurrent fetch per employee
For each employee with both a primary and absence calendar configured, fetch both concurrently. Only the primary calendar failure sets `EmployeeWeekEvents.error`. Absence calendar failures are silently skipped — the calendar is optional and its unavailability should not block the row from rendering.

### Frontend: `CellEvent` kind extended to `"absence"`
`types.ts` adds `"absence"` to the `CellEvent.kind` union. `toCellEvent` maps `Absence` events to `bg-warning/30` color. `TimetableCell` renders absence events using the same non-interactive (`span`) path as bare events, with the warning color applied.

### No changes to settings dialog or ZEP service
The "Abwesenheit" calendar section in `EmployeeIcalDialog` is already fully implemented. No work needed there.

## Risks / Trade-offs

- **Risk**: Absence events from the CalDAV calendar may have varying SUMMARY strings (no standard format) → Accept: render whatever `SUMMARY` the event has; planners recognise their own calendar entries.
- **Risk**: Employees with no absence calendar configured are common initially → Mitigation: `zep_absence_calendar: None` is explicitly handled as "skip silently."
- **Trade-off**: Absence fetch failures are silent → acceptable because the calendar is optional; a failed optional source should not block the primary planning view.
