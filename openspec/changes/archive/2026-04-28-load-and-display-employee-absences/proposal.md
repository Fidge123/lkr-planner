## Why

Planners currently have no way to see when employees are absent (Urlaub, Krankenstand, etc.) in the week view, making it easy to accidentally schedule assignments on unavailable days. Adding absence visibility directly in the planning grid prevents scheduling conflicts and reduces manual cross-checking.

## What Changes

- Add a ZEP absence calendar URL field to each employee's settings (alongside the existing primary calendar)
- Fetch absence events from that CalDAV calendar per employee when loading a week
- Display absence days visually distinct from assignments in the timetable (dedicated cell styling)
- Extend the settings dialog to allow configuring the absence calendar URL per employee

## Capabilities

### New Capabilities
- `employee-absence-display`: Load absence events from a per-employee ZEP CalDAV absence calendar and display them in the timetable week view with distinct visual styling

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: Extend `EmployeeSetting` in local store with `zep_absence_calendar` field; extend `load_week_events` or add new Tauri command; update `TimetableRow`/`TimetableCell` frontend components
- APIs: ZEP CalDAV (same mechanism as existing primary calendar)
- No new external dependencies required
