## Context

`load_week_events` fetches every employee's primary and absence calendar, four employees at a time, and only then resolves every distinct Daylite project of the week, six at a time, before it returns anything.
Each Daylite command builds its own `reqwest::Client`, so no connection outlives the command that opened it, and the project lookups of a week open up to six fresh TLS connections.
At startup the planning grid loads the displayed week on mount, the contact sync then reloads the same week with its cache invalidated, and both adjacent weeks are prefetched, while the first Daylite call refreshes the access token and every other Daylite call waits for it.
Every write from the modal, the quick-add suggestion, or a drag reloads the whole displayed week with the loading spinner shown.

## Goals / Non-Goals

**Goals:**
- Paint a week as soon as its CalDAV events arrive, without waiting for Daylite.
- Keep connections to Daylite open across commands.
- Show project names, statuses, and categories from a previous session immediately after a restart.
- Load the displayed week once at startup unless the contact sync changed a calendar.
- Reflect a drop or a delete in the grid immediately and refresh only the rows an edit touched.
- Make telemetry separate user-facing waits from background work.

**Non-Goals:**
- No backend cache for CalDAV events; that remains the scope of `caldav-caching-improvements`.
- No change to how writes are performed or to the slot allocation they trigger.
- No optimistic placeholder for a newly created assignment; it appears with the targeted refresh that follows the write.

## Decisions

### Two-phase week loading
`load_week_events` resolves project details only from the project cache and never calls Daylite.
An assignment whose reference has no cache entry is returned with its event summary as the title and `project_pending: true`.
The frontend then calls `resolve_week_projects` with the week's project references and patches the returned details into the week, clearing the pending flag of every reference it asked for.
A reference that cannot be resolved comes back as `null`, which leaves the card unresolved with its warning, as before.
Alternative considered: keep one command and emit a Tauri event when the projects arrive.
Rejected because a request and response pair is simpler to test and to tie to the week it belongs to.

### Persisted stale-while-revalidate project cache
An entry stays fresh for five minutes, measured on a monotonic clock.
A stale entry is still served by `load_week_events` and is replaced when `resolve_week_projects` refreshes it; when the refresh fails, the stale entry is returned instead of an error.
Resolved projects are written to `daylite-projects.json` in the app cache directory after a resolution that fetched anything, and read back at startup as stale entries, so wall-clock changes cannot make a restored entry look fresh.
The reload menu passes `force_refresh` to bypass freshness.
The fixed-appointment guard keeps reading the project directly from Daylite.
Alternative considered: keep the 30 second in-memory cache.
Rejected because every restart and every revisit after 30 seconds paid for a full round of project lookups before the week appeared.

### Shared Daylite client
`DayliteApiClient::new` takes a clone of one process-wide `reqwest::Client`, which shares its connection pool.
The ZEP calendar discovery and probes use the existing shared CalDAV client the same way.

### Startup sequence
The frontend reads the local store first, so the week start and the employee settings are known before the first week load.
After the contact sync it reads the store again and reloads the week only if an employee's primary or absence calendar differs from the first read.

### Optimistic edits with targeted refresh
A drop moves the card into its target cell before the write starts, clearing its times until the backend has re-slotted the day.
A delete removes the card once the backend confirms it.
After any write, `load_week_events` is called for the affected employees only, without the loading spinner, and their rows are replaced.
A refresh result is dropped if another optimistic edit happened after it was requested, so an older refresh cannot undo a newer edit.
A failed write also triggers the refresh, which puts the card back where the calendar has it.

### Telemetry
`load_week_events` takes a purpose: `display` keeps the `load_week_events` operation, `prefetch` records `prefetch_week_events`, and `refresh` records `refresh_week_events`.
`resolve_week_projects` records its own operation, and its client records each lookup as `daylite_request`.

## Risks / Trade-offs

- [A project renamed in Daylite shows its old name for up to five minutes] → The reload menu bypasses the cache, and the fixed-appointment guard always reads Daylite.
- [The persisted cache holds project names on disk] → The same names already sit in the calendar events and in the local store's contact cache; the file lives in the app cache directory.
- [A dropped card briefly shows no times] → The targeted refresh follows the write within a few hundred milliseconds.
- [Overlapping edits can briefly show an intermediate state] → The last write's refresh always applies, because it is requested after the last edit.

## Migration Plan

- No data migration: the project cache file is created on first use and ignored when unreadable.
- Reverting the change restores the previous single-phase load.

## Open Questions

None.
