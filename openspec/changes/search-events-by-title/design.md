## Context

The only event read path today is `load_week_events` (`src-tauri/src/integrations/calendar/commands.rs`), which fans out over every configured employee's primary and absence calendar for one week.
Under it, `fetch_events_in_range` (`src-tauri/src/integrations/calendar/caldav/report.rs`) already issues a CalDAV `REPORT` for an arbitrary date range; `fetch_calendar_events` is only the week-shaped wrapper around it.
A search across many weeks therefore needs a new command, not a new transport.

Titles reach the backend as the VEVENT `SUMMARY`; the Daylite project reference is carried separately in the `DESCRIPTION` as `daylite:/v1/projects/42`.
This change searches titles only, so the Daylite side is untouched and no project resolution is needed for a result.

The week the grid shows is `weekOffset` state in `src/app.tsx`, and the only navigation the grid exposes is `onNavigateWeek(direction: -1 | 1)`.

## Goals / Non-Goals

**Goals:**
- Find events by a title fragment across a bounded range of past and future weeks.
- Jump the grid to the week of a chosen result.
- Report per-employee calendar failures without discarding the results that did succeed.

**Non-Goals:**
- No search by Daylite project reference. A rename in Daylite will therefore split a project's history across two titles, which is accepted for this change.
- No highlighting of the matching cards after the jump. `highlight-matching-events` owns card marking and stays independent.
- No absence search. Absence calendars are not requested.
- No search as the planner types, no result paging, and no widening of the range from within the modal.
- No caching of search results.

## Decisions

### Filter titles in Rust, not on the CalDAV server
Issue one `REPORT` per employee primary calendar carrying only the `time-range` filter already in `build_report_body`, and match titles in Rust over the parsed events.

Alternative considered: adding a `prop-filter` on `SUMMARY` with a `text-match` so the server filters.
Rejected on two counts.
Its default collation `i;ascii-casemap` does not case-fold beyond ASCII, so a search for `müller` would miss `Müller` in an application whose data is German throughout; `i;unicode-casemap` is not reliably implemented.
And a server that does not implement `prop-filter` returns the unfiltered calendar rather than an error, so a broken filter is indistinguishable from a working one until the results are wrong.
Matching locally costs one range response per employee, which is the same request count either way, and makes the matching rule ours to test.

### Case-insensitive substring matching
A title matches when the query, trimmed and lowercased, is contained in the title lowercased, using Rust's Unicode-aware `to_lowercase`.
An empty or whitespace-only query is not searched.

Alternative considered: word-prefix or fuzzy matching.
Rejected as unneeded for titles that are short project names, and as a rule that is harder to explain than "contains".

### Search on submit, not while typing
The modal searches when the planner submits the field.
`useAssignmentProjectSearch` debounces and requires three characters because it queries Daylite on every keystroke; a search that fans out over every employee's year of calendar data must not fire on a keystroke at all.
Submitting also removes the minimum-length rule: a deliberate two-character search is the planner's to make.

### Fixed range of one year back and one year forward
The backend derives the range from today.

Alternative considered: an unbounded query, by omitting `time-range`.
Rejected because it makes the response size a property of the server's history rather than of the request.

Alternative considered: a range the planner widens from the modal.
Rejected for this change as scope that is only worth adding once the fixed range is shown to be too narrow in use.
The modal states the searched range so a miss is not read as an absence of the event.

### The frontend chooses which employees are searched
The command takes the employee references to search, and the frontend passes exactly the employees the grid is currently showing.

Alternative considered: searching every employee with a configured primary calendar, as `load_week_events` does.
Rejected because a result for an employee hidden by `hideNonPlannableEmployees` would jump the grid to a week where the matching card is not rendered.
Keeping the searched set equal to the displayed set means every result is reachable.

### Absolute week jump
Add a helper to `src/app/util.ts` that turns a date into the `weekOffset` whose week contains it, and let the modal set `weekOffset` directly in `src/app.tsx`.
`onNavigateWeek` steps by one week and is kept as is for the arrow and swipe navigation it already serves.

### Per-employee errors
The command returns the hits it has plus the employees it could not read, mirroring how `load_week_events` reports a per-employee failure rather than failing the whole load.
The modal names the failed employees in German above the results.

## Risks / Trade-offs

- [A year of events per employee in one response, where the grid fetches a week] → The fan-out is one request per employee, not one per week, and a search is a deliberate, occasional action rather than something navigation triggers. The modal shows a loading state while it runs.
- [A project renamed in Daylite is findable only under the title stored on each event] → Accepted. The events carry the title they were written with, and searching by project reference is deliberately out of scope here.
- [The searched range silently excludes older history] → Mitigated by stating the range in the modal, so an empty result reads as "not in this range" rather than "not scheduled".

## Relationship to `caldav-caching-improvements`

The two changes are independent, and neither blocks the other.

That change caches resolved week events keyed by `(employee_id, calendar_url, week_start)`, to spare repeated identical fetches during week navigation.
A search does not fit that cache in either direction.
It reads a range that is not a week, so it cannot look entries up; and its results are a title-filtered subset, so writing them under a week key would leave the grid rendering a week with most of its events missing.
Search therefore bypasses the cache entirely, and the code must not be tempted to share the key space.

The one piece of that change search does depend on has already landed outside it: the shared `caldav_client()` reached through `build_caldav_session`, which gives the search fan-out connection reuse without any further work.

## Open Questions

None.
