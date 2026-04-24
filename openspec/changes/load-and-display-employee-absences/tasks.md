## 1. Backend: Absence Event Kind

- [ ] 1.1 Add `Absence` variant to `CalendarEventKind` enum in `calendar.rs`
- [ ] 1.2 Update any exhaustive matches on `CalendarEventKind` to handle the new variant

## 2. Backend: Fetch Absence Events in `load_week_events`

- [ ] 2.1 For each employee with `zep_absence_calendar` set, fetch CalDAV events from that URL using the existing `fetch_calendar_events` function
- [ ] 2.2 Fetch primary and absence calendars concurrently per employee
- [ ] 2.3 Map fetched absence events to `CalendarCellEvent` with `kind: Absence` and no `project_status`
- [ ] 2.4 Append absence events to the employee's `events` vec (absence calendar fetch failures are silently skipped; do not set `error`)

## 3. Backend: Testing

- [ ] 3.1 Write unit test: absence events included in response when absence calendar is configured
- [ ] 3.2 Write unit test: no absence events when `zep_absence_calendar` is `None`
- [ ] 3.3 Write unit test: absence calendar fetch failure does not affect primary calendar events or `error` field
- [ ] 3.4 Write unit test: absence event has `kind: Absence`, correct `title` from SUMMARY, and `project_status: None`

## 4. Frontend: Type and Mapping

- [ ] 4.1 Add `"absence"` to the `CellEvent.kind` union in `types.ts`
- [ ] 4.2 Map `Absence` events to `bg-warning/30` color in `toCellEvent`

## 5. Frontend: Cell Rendering

- [ ] 5.1 Add an absence rendering branch to `TimetableCell`: non-interactive `span` with warning color styling
