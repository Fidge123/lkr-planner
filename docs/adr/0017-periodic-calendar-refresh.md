# ADR 0017: Periodic Calendar Refresh from the Frontend

- Status: Accepted
- Date: 2026-10-19

## Context

The planning grid fetched a week's assignments only on start, on navigation, on "Daten neu laden" and after writes.
Planners leave the app open in the background for hours or days while colleagues and synced calendars change the CalDAV data, so a returning planner saw outdated appointments without any hint.
CalDAV offers no push channel the app could subscribe to.
ADR 0014 limits the app to macOS and Linux, where the webview throttles the timers of a hidden window and a sleeping machine freezes them.
Right after wake, Wi-Fi and VPN are often still connecting while the first Daylite call of the day rotates the expired access token.
Daylite retires the old refresh token as it issues the new one (ADR 0006), so a rotation response lost on a flapping network signs the user out on the next start.

### Evaluated Options
- Poll the displayed week from the frontend, timed from the age of its data
  - Pros: Reuses `load_week_events` and the frontend's week cache; navigation, manual reloads and failed loads restart the countdown without extra wiring.
  - Cons: Each refresh costs one CalDAV `REPORT` per employee and calendar even when nothing changed.
- Poll from the backend and notify the frontend through a Tauri event
  - Pros: Independent of the webview's timer throttling.
  - Cons: Needs an event channel and duplicates the tracking of the displayed week, which lives in the frontend.
- Detect changes with CalDAV `sync-token` or `getctag` before fetching
  - Pros: Cheaper requests when nothing changed.
  - Cons: Needs per-calendar sync state in the backend and support on every server, for a load that is already negligible at a 5 minute interval.

## Decision

- `useCalendarAutoRefresh` fetches the displayed week again once 5 minutes have passed since its last completed load, while the window is visible and the network is up.
- Each cached week records when it was fetched, so showing a cached week keeps the age of its data.
- A failed load counts as completed, so a failing server is not hit in a loop.
- A visibility change, window focus or `online` event schedules a check 5 seconds later, which refreshes when the data is 5 minutes old or older; several events in a row collapse into one check.
- No automatic refresh starts while `navigator.onLine` is false, and the `online` event triggers the check once the network is back.
- The 5 second settle delay keeps the refresh, and with it a possible Daylite token rotation, out of the first seconds after wake.
- An automatic refresh drops every other cached week and keeps the displayed week on screen while it runs and when it fails.
- An automatic refresh uses the `refresh` load purpose, since nobody waits for it.
- A failed automatic refresh is retried once after 30 seconds without an error, and only a failed retry shows the German error with "Erneut laden".
  Any other load of the displayed week drops the pending retry, and a retry that falls due while offline is skipped.
- An automatic refresh that completes after an optimistic edit made during its request keeps the edited week instead of its own data, which may predate the write.
- Row refreshes after a write (ADR 0016) do not restart the interval, since they fetch only the affected employees.

## Consequences

- A planner returning to the app sees data at most 5 minutes old, or fresh data 5 seconds after the window comes back.
- Navigating after an automatic refresh fetches the target week again, and the adjacent weeks are prefetched again only on the next navigation.
- A network that is still connecting after wake no longer greets the planner with an error, because the silent retry covers it.
- `navigator.onLine` only reports whether an interface is up, so an unreachable VPN or server still runs into the retry and, if that fails too, the error.
- The interval is a constant, not a setting.
