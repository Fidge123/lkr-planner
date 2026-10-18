## Why

A planner who needs to know when a job was last on site, or when it is next scheduled, has no way to ask.
The grid shows one week, and the only way to reach another week is to navigate into it and read it.
Finding a single past appointment means paging backwards a week at a time and scanning every row on the way.

The `highlight-matching-events` change answers the neighbouring question, "where else is this work in the week I am looking at", by picking a card.
It cannot answer "when else", because the data it marks is the data already on screen.
Reaching beyond the visible week takes a query the planner types and a result list that names dates, which is what every calendar application offers and this one does not.

## What Changes

- A search button in the application header opens a search modal.
- The planner types a title fragment and submits it.
  Submitting runs the search; the field does not search while typing.
- The search matches event titles, case-insensitively, anywhere in the title.
  Matching is Unicode-aware, so a query typed without capitals still finds `Müller`.
- The search covers assignments and bare events on the primary calendars of the employees the grid is currently showing.
  Absence calendars are not read at all, so an absence is never a result.
- The searched range is one year back and one year forward from today.
  The modal states the range, so the planner knows what was looked at.
- Results are listed oldest first, each naming its date, weekday, employee, time and title.
- Clicking a result jumps the grid to the week holding that date and closes the modal.
- Employees whose calendar could not be read are reported in the modal by name, and the results found for the others are still shown.

## Capabilities

### New Capabilities

- `event-search`: finding events by title across a range of weeks and jumping the grid to a result's week.

### Modified Capabilities

None.

## Impact

- `src-tauri/src/integrations/calendar/caldav/report.rs`: no change.
  `fetch_events_in_range` already takes an arbitrary range; only the week-shaped wrapper is specific to the grid.
- `src-tauri/src/integrations/calendar/commands.rs`: a new `search_events_by_title` command, fanning out over the given employees' primary calendars with the existing shared `caldav_client()` and the existing concurrency bound.
- `src-tauri/src/integrations/calendar/types.rs`: an `EventSearchHit` type carrying the date, employee reference, title and times of one match.
- `src/generated/tauri.ts`: regenerated bindings for the new command and type.
- `src/app/util.ts`: a helper turning a date into the week offset that shows it, so a result can be jumped to.
- `src/app.tsx`: the search button, the modal's open state, and the absolute week jump.
  `onNavigateWeek` is relative by one week and cannot express the jump.
- `src/app/components/event-search-dialog.tsx`: the modal, its query field, its result list and its error reporting.
- No interaction with `caldav-caching-improvements`.
  That change caches week-shaped reads for the grid; a search reads an arbitrary range and neither reads from nor writes to that cache.
- Tests: `cargo test` for the title matching and the range derivation, `bun test` for the week-offset helper, the result list, the jump and the partial-error reporting.
