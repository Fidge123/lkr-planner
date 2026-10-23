# Proposal

## Why

The planning grid only fetches a week's assignments on start, on week navigation and on a manual "Daten neu laden".
A planner who leaves the app open in the background for hours or days returns to a grid that silently shows outdated appointments, because colleagues and synced calendars changed the CalDAV data in the meantime.

## What Changes

- Refresh the displayed week's assignments automatically every 5 minutes while the window is visible.
- Refresh shortly after the window becomes visible, regains focus or the network comes back when the last refresh is older than the interval, covering timers the operating system throttled or suspended while the app was hidden or the machine slept.
- Skip automatic refreshes while offline and wait 5 seconds after returning, so a refresh does not run on a network that is still connecting after wake, where a lost Daylite token rotation would sign the user out.
- Discard all cached weeks on an automatic refresh, so navigating to a previously visited week fetches it again instead of showing data from before the refresh.
- Keep the displayed week on screen while an automatic refresh runs and when it fails, instead of emptying the grid.
- Retry a failed automatic refresh once after 30 seconds without an error, and show the German error with its "Erneut laden" action only when the retry fails too.
- Manual reloads and week navigation count as refreshes and restart the interval; the row refresh after a write does not, since it fetches only the affected employees.

## Capabilities

### New Capabilities

- `calendar-auto-refresh`: Periodic and on-return refresh of the displayed week's calendar data, including when a refresh is due, which cached weeks it discards and how failures are presented.

### Modified Capabilities

None.

## Impact

- `src/app/hooks/use-planning-assignments.ts`: track the time of the last completed load, discard the whole week cache on an automatic refresh, keep displayed data when an automatic refresh fails and retry it once.
- New hook `src/app/hooks/use-calendar-auto-refresh.ts` with the interval and the visibility, focus and online scheduling, wired up in `src/app.tsx`.
- No backend changes: the refresh reuses `load_week_events`.
- The in-flight `caldav-caching-improvements` change adds a 30 second backend cache, which is far shorter than the refresh interval and therefore does not hide changes from an automatic refresh.
- `docs/adr/`: new ADR recording the choice of client-side polling over server push for keeping CalDAV data fresh.
