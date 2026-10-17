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

**Non-Goals:**

- Reacting to calendar changes made outside the app; they are picked up the next time an app write touches that day.
- Backfilling appointments for days no write touches after this change ships.
- Reading Daylite appointments back into the planning grid.
- Surfacing synchronization failures in the UI.

## Decisions

### Reconcile whole days from desired state

After a write, the backend reconciles each touched day: it reads that day from every configured primary calendar, builds the desired appointment per project, searches Daylite for the app-managed appointments on that day, and applies the difference.
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
- Times: minimum start and maximum end over the project's assignment events of that day; an assignment event without times counts as the fixed 08:00 to 16:00 window.
- Fixed fields on every create and patch: `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, `invitations: []`, `projects: [{"project": <ref>}]`.
- `invitations` is always sent as an empty array, because Daylite may otherwise default the requesting user or keep invitees added later.

A patch is sent only when one of these fields differs from the searched appointment, which is why the search requests `full-records=true` with an explicit `limit=250`, since full records default to a page size of ten.

### Run in the background, serialized per day

The command spawns the reconciliation with `tauri::async_runtime::spawn` after the CalDAV write returns, so the command result and latency do not change.
A process-wide map of per-date `tokio::sync::Mutex` guards serializes reconciliations of the same day, so two quick writes cannot both observe "no appointment" and create duplicates.
Cross-device races remain possible and are healed by the duplicate rule.

### Failure handling

A failure reading any worker's calendar for the day aborts that day's reconciliation before any Daylite write, because removing a name based on a partial view would be wrong.
Daylite request failures are recorded under a new telemetry operation `DayliteAppointmentSync` and do not affect the command result.
A failure for one project does not stop the remaining projects of the day.

### Module layout

- `src-tauri/src/integrations/daylite/appointments.rs`: DTOs and the search, create, patch and delete requests.
- `src-tauri/src/integrations/calendar/appointment_sync.rs`: the pure planner turning a day's events, worker names, project names and existing appointments into a list of create, patch and delete actions, plus the orchestrator that fetches, plans and applies.
- `src-tauri/src/integrations/calendar/commands.rs`: schedules the reconciliation for the touched days after each successful write.

## Risks / Trade-offs

- [Each write reads the day from every worker's calendar] → One REPORT per configured worker at the existing concurrency limit of four, in the background, so the planner never waits on it.
- [Manual edits to an app-managed appointment's subject, times, status or invitees are overwritten] → Intended, since the calendar is the source of truth; other fields such as notes, details and extra keywords are left untouched.
- [Daylite `DELETE` only moves the appointment to the trash] → Trashed appointments are excluded from search by default, so a re-added assignment creates a fresh appointment instead of restoring the trashed one.
- [Background work outlives a quickly closed app] → An unfinished reconciliation converges on the next write to that day.
- [The keyword appears in Daylite's keyword list] → It doubles as a visible hint that the appointment is maintained by the app.

## Migration Plan

No data migration.
Existing assignments gain appointments the first time a write touches their day.
Rollback is removing the reconciliation call; appointments already created stay in Daylite and can be found by the `lkr-planner` keyword.

## Open Questions

- Whether the user wants a German toast when a synchronization fails; this can be added later through a Tauri event without changing the reconciliation.
