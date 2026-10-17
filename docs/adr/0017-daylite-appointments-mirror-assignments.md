# ADR 0017: Daylite Appointments Mirror Planned Project Days

- Status: Accepted
- Date: 2026-10-17

## Context

Assignments planned in lkr-planner live only in the workers' CalDAV calendars, so a project's Daylite timeline does not show when and by whom the project was worked on.
Each planned project day should appear there as one Daylite appointment linked to the project, without inviting the workers or anyone else.
Every assignment write re-allocates the slots of the worker's day, so one write can shift the times of other projects on that day.
The CalDAV events use floating wall-clock times without a time zone.
The Daylite search API filters on `keywords`, linked `projects` and `utc_start`, but not on `details`.

### Evaluated Options
- Reconcile each touched day from its desired state: read the day from every worker's primary calendar, build one appointment per project, and apply the difference to Daylite
  - Pros: Covers slot shifts, project changes and moves without tracking what changed; a failed or missed sync heals on the next write to that day.
  - Cons: Every write reads the day from every configured calendar and searches Daylite, in the background.
- Translate each write into an incremental edit, appending or removing one worker's name
  - Pros: One Daylite request per write and no calendar reads.
  - Cons: Needs the previous project and worker of every write, cannot follow the slot re-allocation of neighbouring projects, and never heals a missed update.
- Reconcile only the project a write touched
  - Pros: Fewer Daylite requests than reconciling the whole day.
  - Cons: Misses the other projects on that day whose times the slot re-allocation shifted.
- Mark app-managed appointments with the keyword `lkr-planner`
  - Pros: Searchable, survives edits in Daylite, and doubles as a visible hint that the app maintains the appointment.
  - Cons: Adds an entry to Daylite's keyword list.
- Mark app-managed appointments with text in `details`
  - Pros: Invisible in the keyword list.
  - Cons: `details` is not searchable, so every appointment of the project would have to be fetched and inspected.
- Store the Daylite appointment reference on each assignment event
  - Pros: Direct lookup without a search.
  - Cons: Every worker's event of the project day has to be stamped and the property carried through moves; an appointment whose events were deleted outside the app can no longer be found.
- Write appointments with `timezone: "floating"` and local times
  - Pros: Matches the floating calendar events, and Daylite reports UTC and local times as equal, so a day's search range needs no time zone arithmetic.
  - Cons: An appointment shows the same wall-clock time in every viewer's time zone.
- Write appointments in `Europe/Berlin` with UTC conversion
  - Pros: Appointments carry an explicit time zone.
  - Cons: Needs a time zone database dependency and daylight-saving-aware search ranges for no visible gain.
- Report failed days through a backend registry, a Tauri event and a retry command
  - Pros: A later write that heals a day also clears it from the alert, and command results stay unchanged.
  - Cons: Failed days are forgotten when the app restarts.
- Run the sync inside each command and return its outcome
  - Pros: The frontend learns about a failure from the command it awaited.
  - Cons: Adds Daylite latency to every planner action and changes all five command result types.
- Hold the failed days only in the frontend
  - Pros: No backend state.
  - Cons: A later write that heals a day could not clear it, so the alert would report failures that no longer exist.

## Decision

- After every successful create, update, move, reorder and delete, the backend reconciles the day the write left and the day it entered in a background task, so the command's result and latency do not change.
- A failed CalDAV write schedules no reconciliation.
- A reconciliation reads the day from every configured primary calendar, resolves project names through the cached project resolution with the event summary as fallback, and takes worker names from the contact cache the way the planning grid labels its rows.
- Each project of the day gets one appointment titled `<project> (<names>)`, with names listed once per worker and sorted ignoring case, from the earliest start to the latest end of the project's assignments; an assignment without times counts as 08:00 to 16:00.
- App-managed appointments carry the keyword `lkr-planner`; only appointments with that keyword are searched, updated or removed, and updates never send `keywords`, so keywords added in Daylite survive.
- Every create and patch sends `timezone: "floating"`, local times, `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, the project as the only linked project, and an empty `invitations` list, because Daylite may otherwise invite the requesting user or keep invitees added later.
- A patch is only sent when one of these attributes differs from the searched appointment; among several appointments for one project the lowest Daylite ID is kept and the others are deleted.
- A process-wide lock per date serializes reconciliations of the same day, so two quick writes cannot both create an appointment.
- A day whose calendars cannot all be read is not written to Daylite, because removing a name based on a partial view would be wrong.
- A failure for one project does not stop the others but fails the day; every failure is recorded under the telemetry operation `daylite_appointment_sync`.
- The backend keeps the set of failed days in memory and emits `daylite-appointment-sync-failures` with the sorted ISO dates whenever the set changes.
- The planning grid shows one German error alert naming the failed days and an `Erneut versuchen` button that calls `retry_daylite_appointment_sync`, which reconciles every failed day through the same per-date locks.

## Consequences

- Daylite converges to the calendar state of every day a write touches, including other projects whose slots shifted.
- Calendar entries the app did not write gain an appointment the first time a write touches their day; days no write touches are never backfilled.
- Manual edits to an app-managed appointment's subject, times, status, type or invitees are overwritten on the next sync of its day; notes, details and extra keywords are kept.
- Deleted appointments go to the Daylite trash, which search excludes, so a re-added assignment creates a fresh appointment.
- A reconciliation that the closing app interrupts converges on the next write to that day.
- Failed days are lost on restart and heal on the next write that touches them; telemetry still holds the failure.
- Concurrent writes from two devices can still create duplicates, which the next sync of that day removes.
