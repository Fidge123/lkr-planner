# Design

## Context

Assignments are VEVENTs in each worker's primary CalDAV calendar, written by the create, update, move, reorder and delete commands in `src-tauri/src/integrations/calendar/commands.rs`.
Each VEVENT carries `daylite:<project ref>` in its DESCRIPTION.
The CalDAV events use floating local times without a time zone.
Daylite requests go through `DayliteApiClient` with `with_daylite_tokens`, which retries once after a `401`.
The Daylite API offers `POST /v1/appointments`, `PATCH /v1/appointments/{id}`, `DELETE /v1/appointments/{id}` (moves to trash) and `POST /v1/appointments/_search` with filters on `keywords`, `projects` and `utc_start`.
On a patch, a list field sent as an array replaces the whole list, while `{"add": [...], "remove": [...]}` changes only the given entries.
See proposal.md for the motivation and `specs/daylite-appointment-sync/spec.md` for the required behavior.

## Goals / Non-Goals

**Goals:**

- Keep Daylite in step with an edited worker's calendar while reading only that worker's calendar.
- Keep the assignment commands' latency and results unchanged.
- Report failed days to the user and let them retry without changing the existing commands' results.

**Non-Goals:**

- Re-checking other workers of a project day; their links change only when their own day is edited.
- Reacting to calendar changes made outside the app; they are picked up the next time an app write touches that worker's day.
- Backfilling appointments for worker days no write touches after this change ships.
- Reading Daylite appointments back into the planning grid.
- Remembering failed days across app restarts.

## Decisions

### Synchronize the edited worker's day

After a write, the backend synchronizes each touched worker day: it reads that day from the edited worker's primary calendar, searches Daylite for the app-managed appointments on that day, and changes only that worker's links.
The worker's contact is linked to the appointment of every project the worker holds that day, creating the appointment when the project has none, and unlinked from every appointment of a project the worker no longer holds.
An appointment whose only linked contact is the worker is deleted instead of unlinked, so an appointment to which a contact was linked in Daylite stays.
Among several appointments for one project, the lowest Daylite ID is kept, takes over the other appointments' contacts, and the others are deleted once that update succeeded.

Alternatives considered:

- Reconciling the whole day from every planning contact's calendar: repairs every worker of the day on each write, but reads one calendar per planning contact and fails the day when any of them is unreadable.
- Keeping the worker list in the title: needs the title to be parsed back into workers, which breaks on renamed nicknames and duplicate names.

Contact links identify each worker by reference, so adding and removing one worker needs neither the other workers' calendars nor the title.

### Touched worker days come from the write cores

Create touches the created assignment's worker and date.
Update, move and delete also touch the date the event had before the write, which `update_assignment_core`, `move_assignment_core` and `delete_assignment_core` return alongside their current result.
The worker of an updated, moved or deleted event is the employee whose configured primary calendar holds the event's href, comparing parsed URLs because hand-typed calendar URLs can differ from the server's hrefs in host case, default ports and path encoding; an event in no configured calendar is recorded in telemetry and not synchronized.
A move touches the target worker on the target date and the source worker on the source date, in that order, so the source's unlink cannot delete an appointment the target is joining.
Reordering changes nothing an appointment holds, so it touches nothing.
A failed CalDAV write returns before synchronization is scheduled.

### Identify app-managed appointments by keyword

Created appointments carry the keyword `lkr-planner`, and the search filters on it together with the day's `utc_start` range.
The keyword is a searchable field and survives edits in Daylite, so appointments people create by hand for the same project are never matched.

Alternatives considered:

- Marker text in `details`: `details` is not searchable, so every appointment linked to the project would have to be fetched and inspected.
- Storing the Daylite appointment reference on each VEVENT: would require stamping every worker's event for the project day, carrying the property through moves, and still could not find an appointment whose VEVENT was deleted outside the app.

Updates never send `keywords`, so keywords added in Daylite are preserved.

### Fixed floating times

Appointments are written with `timezone: "floating"` and the local times 08:00 to 16:00 of their day, regardless of the assignment events' times.
For floating appointments Daylite reports UTC and local times as equal, so the search range for a day is `<date>T00:00:00Z` to `<date>T23:59:59Z` without time zone arithmetic.

### Appointment content

- `subject`: `Einsatz <dd.MM.yyyy>`.
- Fixed fields on every create and update: `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, `invitations: []`, `projects: [{"project": <ref>}]`.
- `contacts`: a create links the edited worker; an update links contacts through `add` and an unlink removes the worker through `remove`, so contacts linked in Daylite or by another device are never resent or dropped.
- `invitations` is always sent as an empty array, because Daylite may otherwise default the requesting user or keep invitees added later.

An update is sent only when a link is missing or one of the fixed fields differs from the searched appointment, which is why the search requests `full-records=true` with an explicit `limit=250`, since full records default to a page size of ten.

### Run in the background, serialized per day

The command spawns one task per write with `tauri::async_runtime::spawn` after the CalDAV write returns, so the command result and latency do not change.
The task synchronizes the write's worker days one after the other in the order the write lists them.
A process-wide map of per-date `tokio::sync::Mutex` guards serializes synchronizations of the same day, so two quick writes cannot both observe "no appointment" and create duplicates.
Cross-device races remain possible and are healed by the duplicate rule.

### Failure handling

A failure reading the worker's calendar aborts that worker day's synchronization before any Daylite write, because without it every link of the worker would look stale.
Daylite request failures are recorded under the telemetry operation `DayliteAppointmentSync` and do not affect the command result.
A failure for one appointment does not stop the worker's remaining appointments, but marks the worker day as failed.

### Failure registry, event and retry command

The backend keeps a process-wide set of failed worker days next to the per-date locks.
A synchronization that fails adds its worker day and one that succeeds removes it, so a later write that heals a worker day also clears it.
Every change to the set emits the Tauri event `daylite-appointment-sync-failures` carrying the sorted ISO dates of the failed worker days, while the registry still holds its lock, so two outcomes finishing together reach the frontend in order.
The command `retry_daylite_appointment_sync` synchronizes every failed worker day through the same per-date locks, one at a time because Daylite answers a burst of requests with `429`, and resolves once all of them have finished; the event reports what is still failing.

In the frontend, `src/app/hooks/use-appointment-sync-failures.ts` subscribes to the event through the Tauri event helper it shares with `use-reload-data-menu.ts`, and exposes the failed dates, a `retry` function and an `isRetrying` flag.
`src/app/page.tsx` renders a DaisyUI `alert alert-error` above the planning grid with the German message and an `Erneut versuchen` button that is disabled and reads `Wird wiederholt...` while `isRetrying` is set.
The planning grid subscribes on mount, before any write can run, so no failure event can be missed and no initial query command is needed.

Alternatives considered:

- Running the synchronization inside each command and returning its outcome: adds Daylite latency to every planner action and changes all five command result types.
- Holding the failed dates only in the frontend: a later write that heals a day could not clear it, so the alert would report failures that no longer exist.
- Persisting the failed worker days in the local store: survives restarts, but a stale entry is healed by the next write anyway, which does not justify a local-store schema change.

### Module layout

- `src-tauri/src/integrations/daylite/appointments.rs`: the appointment record, the `AppointmentStore` trait, and the search, create, update, unlink and delete requests.
- `src-tauri/src/integrations/calendar/appointment_sync.rs`: the synchronization that reads the worker's calendar, searches the day's appointments and sends the creates, updates, unlinks and deletes, the per-date locks and the failed worker day registry that emits the event.
- `src-tauri/src/integrations/calendar/commands.rs`: schedules the synchronization for the touched worker days after each successful write and hosts `retry_daylite_appointment_sync`.
- `src/app/hooks/use-appointment-sync-failures.ts` and the alert in `src/app/page.tsx`.

## Risks / Trade-offs

- [Other workers' links are never re-checked] → A link left stale by a failed synchronization or an edit outside the app stays until that worker's day is edited again.
- [An appointment is deleted when the worker is its only linked contact] → If another worker's link was missed earlier, that worker's next edit of the day creates the appointment again.
- [Daylite might link the requesting user to every appointment] → No appointment would ever be deleted; the recorded cassette shows whether Daylite does this.
- [Existing assignments gain links one worker at a time] → An appointment can link only some of the project day's workers until each of them is edited.
- [Manual edits to an app-managed appointment's subject, times, status, invitees or linked projects are overwritten] → Intended; other fields such as notes, details, extra keywords and extra contacts are left untouched.
- [Daylite `DELETE` only moves the appointment to the trash] → Trashed appointments are excluded from search by default, so a re-added assignment creates a fresh appointment instead of restoring the trashed one.
- [Background work outlives a quickly closed app] → An unfinished synchronization converges on the next write to that worker's day.
- [The failed worker days are lost when the app restarts] → They converge on the next write that touches them; telemetry still holds the failure.
- [Daylite stays unreachable] → The alert stays visible and each retry reports the remaining days, so the user knows the appointments are out of date.
- [Every appointment of a day has the same title] → The project is visible through the appointment's link and in the project's timeline.

## Migration Plan

No data migration.
Existing assignments gain links the first time a write touches their worker's day.
Rollback is removing the synchronization call; appointments already created stay in Daylite and can be found by the `lkr-planner` keyword.
