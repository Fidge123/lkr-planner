# Design

## Context

`usePlanningAssignments` keeps a per-session, in-memory cache of loaded weeks and only calls `load_week_events` on navigation, on `reloadAssignments()` (menu reload, "Erneut laden", writes) and for the idle prefetch of the adjacent weeks.
`reloadAssignments()` drops only the active week from the cache and, on failure, clears `eventsByEmployee` so the grid empties behind the error alert.
`useFrozenDuringDrag` in `src/app/page.tsx` already pins the displayed assignments while a drag is active, so any data update that lands mid-drag is deferred until the drop.
CalDAV offers no push channel the app could subscribe to, and ADR 0014 limits the platforms to macOS and Linux, where the webview throttles or suspends timers of hidden windows and of a sleeping machine.

## Goals / Non-Goals

**Goals:**

- A single timestamp of the last completed load of the displayed week drives both the interval and the on-return check, so a manual reload, navigation or write automatically postpones the next automatic refresh.
- An automatic refresh never empties a grid that already shows data.

**Non-Goals:**

- Refreshing Daylite contacts, project category colors, holidays or local store settings; they change rarely and stay on the manual "Daten neu laden".
- Making the interval configurable in the settings dialog.
- Backend-side polling or change detection such as CalDAV `sync-token` or `getctag`.
- Pausing the refresh while the assignment modal is open; the modal works on its own copy of the event and the grid behind it may update.

## Decisions

### Polling from the frontend instead of push or backend polling

The refresh is scheduled in a new hook `useCalendarAutoRefresh` and calls into `usePlanningAssignments`, reusing the existing `load_week_events` command.
Backend polling would need an event channel to the frontend and duplicate the week tracking the frontend already does, and CalDAV has no push mechanism to replace polling.
This is recorded as an ADR.

### Interval of 5 minutes

Each refresh issues one `REPORT` per employee for the primary and the absence calendar plus Daylite project lookups for unknown references.
5 minutes keeps that load negligible for the team size while bounding staleness to a value planners accept for a shared board.
The value is a named constant in the hook, not a setting (YAGNI).

### Timestamp-based scheduling instead of a fixed `setInterval`

Each cached week records when it was fetched, and `usePlanningAssignments` exposes `lastLoadedAt` for the displayed week: the fetch time of the shown data, or the completion time of a failed load.
Showing a week from the cache therefore keeps the age of that cached data instead of resetting the countdown.
The hook schedules a `setTimeout` for `lastLoadedAt + interval` and re-arms whenever `lastLoadedAt` changes, so any load restarts the countdown without extra wiring.
On `visibilitychange` to visible and on window `focus`, the hook compares `Date.now()` with `lastLoadedAt` and refreshes immediately when it is due, which covers timers that were throttled while hidden or frozen during sleep.
When the timer fires while `document.visibilityState` is `hidden`, it does nothing and the next visibility change triggers the check.
Counting a failed load as completed keeps a failing server from being hit in a tight loop.
A fixed `setInterval` was rejected because it would refresh right after a manual reload and would not catch up after sleep.

### Separate automatic refresh entry point in `usePlanningAssignments`

A new `refreshAssignments()` clears the whole week cache, not only the active week, and loads the active week with a flag that preserves the displayed data on failure.
On failure it sets `errorMessage` but leaves `eventsByEmployee`, `errorsByEmployee` and `loadedWeekStart` untouched; on success it clears `errorMessage` as every successful load does.
Adjacent weeks are not prefetched again immediately; the existing prefetch runs on the next navigation as it does today.
Changing `reloadAssignments()` itself to keep data on failure was rejected because its "empty grid on error" behavior is covered by existing tests and specs for the manual reload and is out of scope.

### No special handling for drags and in-flight writes

The existing request id gating in `loadActiveWeek` lets the newest load win, so a refresh that started before a write is superseded by the write's own `reloadAssignments()`.
`useFrozenDuringDrag` already defers a refresh result until the drop, so the hook does not need to know about drags.

## Risks / Trade-offs

- [A refresh lands while the user reads or is about to click a card, and the card moves] → Updates happen at most every 5 minutes and only when the data changed; the drag freeze covers the case where movement would cause a wrong drop.
- [Network not yet available right after wake, so the on-return refresh fails] → The grid keeps its data, the error offers "Erneut laden", and the next automatic attempt follows 5 minutes later.
- [Clearing all cached weeks makes the next navigation slower] → Only one network round-trip per newly visited week, the same cost as the first visit; the prefetch restores instant navigation for adjacent weeks afterwards.
- [The backend cache from `caldav-caching-improvements` serves stale data to a refresh] → Its 30 second TTL is far below the 5 minute interval, so a refresh always reaches CalDAV.
