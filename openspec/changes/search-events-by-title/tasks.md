## 1. Backend title matching

- [ ] 1.1 Write failing `cargo test`s for the title match rule: substring, case-insensitive, non-ASCII case folding, trimmed query, blank query matches nothing
- [ ] 1.2 Implement the match rule in `src-tauri/src/integrations/calendar/events/`, satisfying the tests
- [ ] 1.3 Write failing `cargo test`s for the searched range derived from a given current date, covering the one-year bounds
- [ ] 1.4 Implement the range derivation, satisfying the tests

## 2. Search command

- [ ] 2.1 Add an `EventSearchHit` type in `src-tauri/src/integrations/calendar/types.rs` carrying uid, employee reference, date, start time, end time and title
- [ ] 2.2 Write failing `cargo test`s for assembling a search result from per-employee fetches, covering ascending date order, absences excluded, and per-employee errors kept beside the hits
- [ ] 2.3 Implement `search_events_by_title` in `src-tauri/src/integrations/calendar/commands.rs`, taking the query and the employee references, fanning out over primary calendars only via `fetch_events_in_range` and the shared `caldav_client()`, satisfying the tests
- [ ] 2.4 Register the command in `specta_builder()` and regenerate `src/generated/tauri.ts`

## 3. Week jump helper

- [ ] 3.1 Write failing `bun test`s in `src/app/util.spec.ts` for a helper turning a date into the week offset showing it, covering the current week, past and future weeks, and a Monday and a Sunday boundary
- [ ] 3.2 Implement the helper in `src/app/util.ts`, satisfying the tests

## 4. Search modal

- [ ] 4.1 Write a failing `bun test` that the modal runs no search while the query is typed and runs one on submit, and none for a blank query
- [ ] 4.2 Implement the query field and submit handling in `src/app/components/event-search-dialog.tsx`, satisfying the tests
- [ ] 4.3 Write failing tests for the result list: ascending order, a result naming date, weekday, employee, time and title, an all-day result without a time
- [ ] 4.4 Implement the result list, satisfying the tests
- [ ] 4.5 Write failing tests for the loading state, the no-match message and the stated searched range
- [ ] 4.6 Implement those states, satisfying the tests
- [ ] 4.7 Write failing tests for partial failures: results still shown, failed employees named, a fully failed search reported
- [ ] 4.8 Implement the error reporting, satisfying the tests

## 5. Wiring

- [ ] 5.1 Write a failing test that choosing a result shows that result's week and closes the modal
- [ ] 5.2 Add the header search button and the modal's open state to `src/app.tsx`, and set `weekOffset` from the chosen result, satisfying the test
- [ ] 5.3 Pass the currently displayed employees to the search, so hidden employees are not searched

## 6. Checks

- [ ] 6.1 Run `bun test` and `cargo test`
- [ ] 6.2 Run `bun run lint`
- [ ] 6.3 Run `bunx openspec validate search-events-by-title`
