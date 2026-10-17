## Why

Telemetry reports `load_week_events` at 7 seconds on average, `daylite_list_contacts` at 6 seconds, and `daylite_project_category_colors` at 4.4 seconds, while the same requests take between 140 milliseconds and 1.5 seconds when sent manually.
The difference comes from how the app chains and repeats requests, not from the servers.
`load_week_events` waits for one Daylite request per distinct project before it returns, every Daylite command opens fresh TCP and TLS connections, startup loads the displayed week twice while the token refresh holds up every Daylite call, and every edit reloads the whole week behind a spinner.

## What Changes

- Share one HTTP client across all Daylite commands so connections stay open between commands.
- Split week loading in two: `load_week_events` returns the CalDAV events with project details taken only from the project cache, and a new `resolve_week_projects` command fetches missing or stale project details in the background.
- Keep resolved Daylite projects fresh for five minutes, persist them to disk, and serve an older entry until a refresh replaces it.
- Mark an assignment whose project is still being resolved as pending, so it is not shown as unresolved while the background resolution runs.
- Load the employee settings at startup before the first week load and reload the week after the contact sync only when the reconciled calendar configuration changed.
- Move a dropped card into its target cell immediately, remove a deleted card immediately, and refresh only the affected employees' rows in the background after an edit instead of reloading the whole week behind the spinner.
- Record background week loads under their own telemetry operations, record project resolution as its own operation, and attach telemetry to the project lookup requests.
- Keep the ZEP credentials in memory after the first keychain read, and send calendar discovery and calendar probes through the shared CalDAV client.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `assignment-persistence`: project resolution runs in the background after the week is shown, served from a persisted cache, and the grid reflects the user's own edits without a full reload.
- `daylite-integration`: the project cache keeps entries for five minutes, survives restarts, serves stale entries until a refresh succeeds, and connections are reused across commands.
- `data-reload`: the reload menu also refreshes the project details of the displayed week, bypassing the project cache.
- `appointment-drag-drop`: a drop shows the card in its target cell immediately and refreshes the affected rows in the background.
- `telemetry`: background week loads and project resolution are recorded under their own operation names.

## Impact

- `src-tauri/src/integrations/daylite/client.rs`, `project_cache.rs`, `projects.rs`: shared client, persisted stale-while-revalidate cache, background resolution command.
- `src-tauri/src/integrations/calendar/commands.rs`, `events/resolve.rs`, `types.rs`: cache-only resolution, employee filter, load purpose, `project_pending` flag.
- `src-tauri/src/integrations/zep/`: in-memory credentials, shared CalDAV client.
- `src-tauri/src/integrations/telemetry/events.rs`: new operations.
- `src/app.tsx`, `src/app/hooks/use-planning-assignments.ts`, `use-appointment-drag.ts`, `timetable-row.tsx`, `types.ts`: startup sequence, background enrichment, optimistic edits, targeted refresh.
- `docs/adr/`: ADR 0015 supersedes ADR 0007 and amends the startup sequence of ADR 0011, ADR 0016 records optimistic edits with targeted refresh.
