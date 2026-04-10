## 1. CalDAV Reading Infrastructure (Rust)

- [ ] 1.1 Implement CalDAV REPORT (calendar-query) request with date-range filter
- [ ] 1.2 Parse VEVENT entries from CalDAV REPORT XML response (use iCal parser crate)
- [ ] 1.3 Classify events: lkr-planner (DESCRIPTION first line = `daylite:/<path>`) vs. bare
- [ ] 1.4 Add `load_week_events` Tauri command: takes employee list + weekStart, returns classified events per employee

## 2. Daylite Project Resolution (Rust)

- [ ] 2.1 Resolve project reference via `dayliteCache` in LocalStore (fast path)
- [ ] 2.2 Fall back to Daylite API query if project not found in cache
- [ ] 2.3 Return German placeholder on resolution failure: `"Beschreibung für [SUMMARY] konnte nicht abgerufen werden"`

## 3. Frontend Integration

- [ ] 3.1 Add `usePlanningAssignments(weekStart)` hook calling `load_week_events`
- [ ] 3.2 Update `TimetableRow` to receive pre-computed cell data (remove `getWorkItemsForCell` call)
- [ ] 3.3 Move `PlanningCellProject` type out of `dummy-data.ts` to a dedicated types file
- [ ] 3.4 Remove dummy assignment fixtures from `dummy-data.ts`
- [ ] 3.5 Connect week navigation (`weekOffset`) to `usePlanningAssignments`

## 4. Two-Tier Cell Rendering

- [ ] 4.1 lkr-planner events: colored by project status, show edit affordance (+ button becomes edit)
- [ ] 4.2 Bare events: neutral/grey styling, no edit affordance, read-only
- [ ] 4.3 German loading state while assignment fetch is in progress
- [ ] 4.4 German error state with retry button on CalDAV fetch failure

## 5. Testing

- [ ] 5.1 Rust: VEVENT parsing and classification (lkr-planner vs. bare)
- [ ] 5.2 Rust: project resolution — cache hit, API fallback, placeholder on failure
- [ ] 5.3 UI: lkr-planner events render with correct project color and title
- [ ] 5.4 UI: bare events render with neutral style and no edit affordance
- [ ] 5.5 UI: loading, empty, and error states in German
- [ ] 5.6 UI: week navigation passes correct date range per weekOffset
