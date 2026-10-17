# ADR 0015: Two-Phase Week Loading with a Persisted Daylite Project Cache

- Status: Accepted
- Date: 2026-10-17

## Context

Telemetry reported `load_week_events` at 7 seconds on average, while a single CalDAV `REPORT` for one calendar takes about 180 milliseconds.
The command fetched every employee's calendars and then, before returning anything, resolved every distinct Daylite project of the week with one request per project, six at a time.
Daylite cannot resolve a set of references in one search, so the project phase dominated the load.
ADR 0007 kept resolved projects in memory for 30 seconds, so every restart and every revisit of a week after 30 seconds paid for that phase again.
Every Daylite command also built its own HTTP client, so each one opened fresh TCP and TLS connections.
At startup the planning grid loaded the displayed week, ADR 0011's contact sync loaded it a second time, and the adjacent weeks were prefetched on top.

### Evaluated Options
- Return the CalDAV events with project details from the cache only and resolve the rest in a second command
  - Pros: The week appears as soon as CalDAV answers; the project phase moves off the critical path; each phase is measurable on its own.
  - Cons: An assignment can be shown before its project is known, which needs a pending state the frontend has to respect.
- Keep one command but raise the concurrency limit for project lookups
  - Pros: No contract change.
  - Cons: Daylite answers bursts with HTTP 429, and the week still waits for the slowest lookup.
- Emit a Tauri event when the projects of a week arrive
  - Pros: The backend can push updates without a second request.
  - Cons: Harder to tie a late event to the week and request it belongs to, and harder to test than a request and its response.

## Decision

- `load_week_events` resolves project details only from the project cache and never calls Daylite.
  An assignment whose reference is not cached carries `project_pending: true` and its event summary as the title.
- A new `resolve_week_projects` command resolves a week's references through the cache and returns `null` for a reference neither Daylite nor the cache could resolve.
  The frontend calls it after every displayed, cached, and prefetched week and patches the results in without the loading spinner.
- Project cache entries stay fresh for five minutes on a monotonic clock.
  A stale entry is still served, and a failed refresh returns it instead of an error.
- Resolved projects are written to `daylite-projects.json` in the app cache directory after a resolution that fetched anything, and read back at startup as stale entries, so a wall-clock change cannot make a restored entry look fresh.
- The reload menu passes `force_refresh`, which bypasses freshness; the fixed-appointment guard keeps reading Daylite directly.
- All Daylite commands share one HTTP client, and so do all ZEP CalDAV requests, so connections stay open between commands.
- At startup the frontend reads the local store before the first week load, and reloads the week after the contact sync of ADR 0011 only when an employee's primary or absence calendar changed.
- This supersedes ADR 0007 and amends the startup sequence of ADR 0011.

## Consequences

- A week appears after its CalDAV requests, and after a restart its cards show the project names and colors of the previous session immediately.
- A project renamed or recategorized in Daylite shows its old details for up to five minutes, unless the user reloads.
- Project names now persist on disk in the app cache directory, next to the contact cache the local store already holds.
- A pending assignment shows its summary without the unresolved warning and cannot be dragged until its project is resolved.
- Telemetry measures the CalDAV phase as `load_week_events` and the Daylite phase as `resolve_week_projects`.
