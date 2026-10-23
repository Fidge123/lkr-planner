# Design

## Context

`usePlanningAssignments` keeps a per-session, in-memory cache of loaded weeks and only calls `load_week_events` on navigation, on `reloadAssignments()` (menu reload, "Erneut laden"), for the idle prefetch of the adjacent weeks and, after a write, for the rows of the affected employees (ADR 0016).
`reloadAssignments()` drops only the active week from the cache and, on failure, clears `eventsByEmployee` so the grid empties behind the error alert.
`useFrozenDuringDrag` in `src/app/page.tsx` already pins the displayed assignments while a drag is active, so any data update that lands mid-drag is deferred until the drop.
CalDAV offers no push channel the app could subscribe to, and ADR 0014 limits the platforms to macOS and Linux, where the webview throttles or suspends timers of hidden windows and of a sleeping machine.

## Goals / Non-Goals

**Goals:**

- A single timestamp of the last completed load of the displayed week drives both the interval and the on-return check, so a manual reload or navigation automatically postpones the next automatic refresh.
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
A row refresh after a write fetches only the affected employees, so it leaves the fetch time of the week unchanged.
Showing a week from the cache therefore keeps the age of that cached data instead of resetting the countdown.
The hook schedules a `setTimeout` for `lastLoadedAt + interval` and re-arms whenever `lastLoadedAt` changes, so any load restarts the countdown without extra wiring.
On `visibilitychange` to visible, on window `focus` and on the window `online` event, the hook schedules a due check 5 seconds later that compares `Date.now()` with `lastLoadedAt` and refreshes when it is due, which covers timers that were throttled while hidden or frozen during sleep.
The `online` event matters most after sleep: macOS often sends no `focus` event when the window was focused before the lid closed, but the network drops during sleep and reports back on wake.
When the timer or a due check fires while `document.visibilityState` is `hidden` or `navigator.onLine` is `false`, it does nothing and the next visibility, focus or `online` event triggers the check again.
Counting a failed load as completed keeps a failing server from being hit in a tight loop.
A fixed `setInterval` was rejected because it would refresh right after a manual reload and would not catch up after sleep.

### Settle delay after returning

The due check after a return event waits 5 seconds instead of running at once.
Right after wake, Wi-Fi and VPN are often still connecting, and the first Daylite call of the morning also rotates the expired access token.
Daylite retires the old refresh token as it issues the new one (ADR 0006), so a rotation response lost on a flapping network signs the user out on the next start.
Waiting for `navigator.onLine` plus the settle delay moves that rotation out of the most fragile seconds.
The silent retry below is skipped as well when it falls due while offline, and the `online` event brings the refresh back.
`navigator.onLine` only knows whether an interface is up, not whether a VPN or the servers are reachable, which the retry below covers.

### One silent retry after a failed automatic refresh

A failed automatic refresh schedules one retry 30 seconds later and keeps its error out of `errorMessage`.
Only a failed retry sets `errorMessage` and `lastLoadedAt`, so the next regular attempt follows 5 minutes later.
Any other load of the displayed week, by navigation or reload, clears the pending retry when it starts, since its result supersedes the retry's.
Showing the error on the first failure was rejected because a network that is still connecting after wake would greet the planner with an error every morning.
Retrying more than once was rejected because the regular interval already acts as the next retry.

### Separate automatic refresh entry point in `usePlanningAssignments`

A new `refreshAssignments()` clears every other cached week and fetches the active week bypassing the cache, with a flag that preserves the displayed data on failure.
The active week stays cached until the fetch succeeds, so optimistic edits and row refreshes still have a week to patch after a failed refresh.
On failure it leaves `eventsByEmployee`, `errorsByEmployee` and `loadedWeekStart` untouched; on success it clears `errorMessage` as every successful load does.
It loads with the `refresh` purpose, which telemetry records apart from loads a user waits for.
The hook's cache and load logic moves into a framework-free `createWeekLoader`, so these rules are testable with `bun test` without a DOM.
Adjacent weeks are not prefetched again immediately; the existing prefetch runs on the next navigation as it does today.
Changing `reloadAssignments()` itself to keep data on failure was rejected because its "empty grid on error" behavior is covered by existing tests and specs for the manual reload and is out of scope.

### Edits made while a refresh runs win over its result

Writes refresh only the affected rows (ADR 0016) and do not bump the request id of `loadActiveWeek`, so a refresh that started before a drop could land after the drop's row refresh and put the card back in its old cell.
An automatic refresh therefore keeps the cached week when an optimistic edit happened after it was requested, using the existing edit sequence, and the edit's own row refresh brings the confirmed rows.
`useFrozenDuringDrag` already defers a refresh result until the drop, so the hook does not need to know about drags.

## Risks / Trade-offs

- [A refresh lands while the user reads or is about to click a card, and the card moves] → Updates happen at most every 5 minutes and only when the data changed; the drag freeze covers the case where movement would cause a wrong drop.
- [Network not yet available right after wake, so the on-return refresh fails] → The check waits for `navigator.onLine` and 5 seconds of settling, a failure is retried once after 30 seconds without an error, and only then does the error with "Erneut laden" appear while the grid keeps its data.
- [A Daylite token rotation runs while the network flaps and its response is lost, signing the user out] → The settle delay keeps automatic refreshes out of the first seconds after wake; the remaining risk is the same as for any manual reload on a bad network.
- [The machine wakes without any visibility, focus or `online` event] → The frozen timer resumes and fires at most 5 minutes after wake, because timers count awake time only.
- [Clearing all cached weeks makes the next navigation slower] → Only one network round-trip per newly visited week, the same cost as the first visit; the prefetch restores instant navigation for adjacent weeks afterwards.
- [The backend cache from `caldav-caching-improvements` serves stale data to a refresh] → Its 30 second TTL is far below the 5 minute interval, so a refresh always reaches CalDAV.
