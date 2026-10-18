# Design

## Context

Assignments are VEVENTs in each worker's primary CalDAV calendar, written by the create, update, move, reorder and delete commands in `src-tauri/src/integrations/calendar/commands.rs`.
Each VEVENT carries `daylite:<project ref>` in its DESCRIPTION and the project name in its SUMMARY.
Every write re-allocates the day's slots (`slot-allocation`), so one write can shift the times of other projects on the same worker's day.
The CalDAV events use floating local times without a time zone.
Daylite requests go through `DayliteApiClient` with `with_daylite_tokens`, which retries once after a `401`; worker names are available from the contact cache in the local store.
The Daylite API offers `POST /v1/appointments`, `PATCH /v1/appointments/{id}` (list fields are replaced when sent as arrays), `DELETE /v1/appointments/{id}` (moves to trash) and `POST /v1/appointments/_search` with filters on `keywords`, `projects` and `utc_start`.
See proposal.md for the motivation and `specs/daylite-appointment-sync/spec.md` for the required behavior.

## Goals / Non-Goals

**Goals:**

- Converge Daylite to the calendar state of each touched day with one idempotent reconciliation, instead of translating each write into incremental edits.
- Keep the assignment commands' latency and results unchanged.
- Report failed days to the user and let them retry without changing the existing commands' results.

**Non-Goals:**

- Reacting to calendar changes made outside the app; they are picked up the next time an app write touches that day.
- Backfilling appointments for days no write touches after this change ships.
- Reading Daylite appointments back into the planning grid.
- Remembering failed days across app restarts.

## Decisions

### Reconcile whole days from desired state

After a write, the backend reconciles each touched day: it reads that day from the primary calendar of every planning contact the grid lists, Monteur and Test alike, builds the desired appointment per project, searches Daylite for the app-managed appointments on that day, and applies the difference.
Matching is by linked project reference; a desired project without an appointment is created, an appointment without a desired project is deleted, and a pair that differs is patched.
Among several appointments for one project, the lowest Daylite ID is kept and the rest are deleted.

Alternatives considered:

- Incremental edits per write (append or remove one name): needs the previous project and worker of every write, cannot follow slot re-allocation of neighbouring projects, and never heals a missed update.
- Reconciling only the written project: misses the other projects on the same day whose times the slot re-allocation shifted.

Reconciling the whole day covers slot shifts, project changes and moves without tracking what changed, and a failed sync heals on the next write to that day.

### Touched days come from the write cores

Create and reorder touch the written date.
Update, move and delete also touch the date the event had before the write, which `update_assignment_core`, `move_assignment_core` and `delete_assignment_core` already read; they will return it alongside their current result so the command layer knows both days.
A failed CalDAV write returns before reconciliation is scheduled.

### Identify app-managed appointments by keyword

Created appointments carry the keyword `lkr-planner`, and the search filters on it together with the day's `utc_start` range.
The keyword is a searchable field and survives edits in Daylite, so appointments people create by hand for the same project are never matched.

Alternatives considered:

- Marker text in `details`: `details` is not searchable, so every appointment linked to the project would have to be fetched and inspected.
- Storing the Daylite appointment reference on each VEVENT: would require stamping every worker's event for the project day, carrying the property through moves, and still could not find an appointment whose VEVENT was deleted outside the app.

Updates never send `keywords`, so keywords added in Daylite are preserved.

### Floating times

Appointments are written with `timezone: "floating"` and `local_start` / `local_end` built from the VEVENT's floating wall-clock times.
For floating appointments Daylite reports UTC and local times as equal, so the search range for a day is `[<date>T00:00:00Z, <next date>T00:00:00Z)` without time zone arithmetic.

Alternative considered: `Europe/Berlin` with UTC conversion, which needs a time zone database dependency and DST-aware search ranges for no visible gain, since the calendar events themselves are floating.

### Appointment content

- `subject`: `<project name> (<names>)`, names deduplicated per worker and sorted alphabetically, ignoring case.
- Project name: the Daylite project name from the cached project resolution, falling back to the SUMMARY of the first assignment event when resolution fails.
- Worker name: the contact cache's nickname, then full name, then `Unbenannter Kontakt`, matching the planning grid row label.
- Times: minimum start and maximum end over the project's assignment events of that day; an assignment event without times counts as the fixed 08:00 to 16:00 window, and one that does not end after it starts runs until midnight, because Daylite rejects an appointment that ends before it starts.
- Fixed fields on every create and patch: `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, `invitations: []`, `projects: [{"project": <ref>}]`.
- `invitations` is always sent as an empty array, because Daylite may otherwise default the requesting user or keep invitees added later.

A patch is sent only when one of these fields differs from the searched appointment, which is why the search requests `full-records=true` with an explicit `limit=250`, since full records default to a page size of ten.

### Run in the background, serialized per day

The command spawns the reconciliation with `tauri::async_runtime::spawn` after the CalDAV write returns, so the command result and latency do not change.
A process-wide map of per-date `tokio::sync::Mutex` guards serializes reconciliations of the same day, so two quick writes cannot both observe "no appointment" and create duplicates.
Cross-device races remain possible and are healed by the duplicate rule.

### Failure handling

A failure reading any worker's calendar for the day aborts that day's reconciliation before any Daylite write, because removing a name based on a partial view would be wrong.
A calendar the server reports as not found (`404`) cannot hold assignments, so it counts as empty instead; otherwise a placeholder calendar URL on a Test contact would fail every day.
A calendar still configured for a contact that is no longer a planning contact is not read at all.
Daylite request failures are recorded under a new telemetry operation `DayliteAppointmentSync` and do not affect the command result.
A failure for one project does not stop the remaining projects of the day, but marks the whole day as failed.

### Failure registry, event and retry command

The backend keeps a process-wide set of failed dates next to the per-date locks.
A reconciliation that fails adds its date and one that succeeds removes it, so a later write that heals a day also clears it.
Every change to the set emits the Tauri event `daylite-appointment-sync-failures` carrying the sorted ISO dates.
The new command `retry_daylite_appointment_sync` reconciles every date in the set through the same per-date locks and resolves once all of them have finished; the event reports what is still failing.

The registry announces a change while it still holds its lock, so two outcomes finishing together reach the frontend in order.
A retry reconciles at most two days at once, because Daylite answers a burst of requests with `429`.

In the frontend, `src/app/hooks/use-appointment-sync-failures.ts` subscribes to the event through the Tauri event helper it shares with `use-reload-data-menu.ts`, and exposes the failed dates, a `retry` function and an `isRetrying` flag.
`src/app/page.tsx` renders a DaisyUI `alert alert-error` above the planning grid with the German message and an `Erneut versuchen` button that is disabled and reads `Wird wiederholt...` while `isRetrying` is set.
The planning grid subscribes on mount, before any write can run, so no failure event can be missed and no initial query command is needed.

Alternatives considered:

- Running the synchronization inside each command and returning its outcome: adds Daylite latency to every planner action and changes all five command result types.
- Holding the failed dates only in the frontend: a later write that heals a day could not clear it, so the alert would report failures that no longer exist.
- Persisting the failed dates in the local store: survives restarts, but a stale entry is healed by the next write anyway, which does not justify a local-store schema change.

### Module layout

- `src-tauri/src/integrations/daylite/appointments.rs`: DTOs and the search, create, patch and delete requests.
- `src-tauri/src/integrations/calendar/appointment_sync.rs`: the pure planner turning a day's events, worker names, project names and existing appointments into a list of create, patch and delete actions, plus the orchestrator that fetches, plans and applies.
- `src-tauri/src/integrations/calendar/commands.rs`: schedules the reconciliation for the touched days after each successful write and hosts `retry_daylite_appointment_sync`.
- `src-tauri/src/integrations/calendar/appointment_sync.rs` also holds the per-date locks and the failed-date registry that emits the event.
- `src/app/hooks/use-appointment-sync-failures.ts` and the alert in `src/app/page.tsx`.

## Risks / Trade-offs

- [Each write reads the day from every worker's calendar] → One REPORT per configured worker at the existing concurrency limit of four, in the background, so the planner never waits on it.
- [Manual edits to an app-managed appointment's subject, times, status or invitees are overwritten] → Intended, since the calendar is the source of truth; other fields such as notes, details and extra keywords are left untouched.
- [Daylite `DELETE` only moves the appointment to the trash] → Trashed appointments are excluded from search by default, so a re-added assignment creates a fresh appointment instead of restoring the trashed one.
- [Background work outlives a quickly closed app] → An unfinished reconciliation converges on the next write to that day.
- [The failed-day set is lost when the app restarts] → The day converges on the next write that touches it; telemetry still holds the failure.
- [Daylite stays unreachable] → The alert stays visible and each retry reports the remaining days, so the user knows the appointments are out of date.
- [The keyword appears in Daylite's keyword list] → It doubles as a visible hint that the appointment is maintained by the app.

## Migration Plan

No data migration.
Existing assignments gain appointments the first time a write touches their day.
Rollback is removing the reconciliation call; appointments already created stay in Daylite and can be found by the `lkr-planner` keyword.
