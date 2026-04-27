## 1. Backend: Absence Event Kind

- [ ] 1.1 Write unit test: absence event has `kind: Absence`, correct `title` from SUMMARY, and `project_status: None`
- [ ] 1.2 Add `Absence` variant to `CalendarEventKind` enum in `calendar.rs`
- [ ] 1.3 Update any exhaustive matches on `CalendarEventKind` to handle the new variant

## 2. Backend: Fetch Absence Events in `load_week_events`

- [ ] 2.1 Write unit test: absence events included in response when absence calendar is configured
- [ ] 2.2 Write unit test: no absence events when `zep_absence_calendar` is `None`
- [ ] 2.3 Write unit test: absence calendar fetch failure does not affect primary calendar events or `error` field
- [ ] 2.4 For each employee with `zep_absence_calendar` set, fetch CalDAV events from that URL using the existing `fetch_calendar_events` function
- [ ] 2.5 Map fetched absence events to `CalendarCellEvent` with `kind: Absence` and no `project_status`
- [ ] 2.6 Append absence events to the employee's `events` vec (absence calendar fetch failures are silently skipped; do not set `error`)

## 3. Backend: Multi-Day Absence Expansion

- [ ] 3.1 Write unit test: `parse_ical_events` captures `DTEND` as a date for all-day events
- [ ] 3.2 Write unit test: multi-day all-day absence expands into one event per covered day within the queried week
- [ ] 3.3 Write unit test: multi-day absence starting before the queried week only produces events for days within the week
- [ ] 3.4 Add `dtend: Option<NaiveDate>` to `RawVEvent`; populate it from the iCal `DTEND` date value for all-day events in `parse_ical_events`
- [ ] 3.5 Expand all-day absence events with a `dtend` into one `CalendarCellEvent` per day in `[dtstart, dtend)` clamped to the requested week

## 4. Backend: Concurrent Fetch (Refactor)

- [ ] 4.1 Fetch primary and absence calendars concurrently per employee

## 5. Frontend: Type and Mapping

- [ ] 5.1 Add `"absence"` to the `CellEvent.kind` union in `types.ts`
- [ ] 5.2 Map `Absence` events to `bg-warning/30` color in `toCellEvent`

## 6. Frontend: Cell Rendering

- [ ] 6.1 Add an absence rendering branch to `TimetableCell`: non-interactive `span` with warning color styling
