# ADR 0017: Daylite Appointments Mirror Planned Project Days

- Status: Accepted
- Date: 2026-10-18

## Context

Assignments planned in lkr-planner live only in the workers' CalDAV calendars, so a project's Daylite timeline does not show when and by whom the project was worked on.
Each planned project day should appear there as one Daylite appointment linked to the project and its workers, without inviting anyone.
Every assignment write re-allocates the slots of the worker's day, and the CalDAV events use floating wall-clock times without a time zone.
The Daylite search API filters on `keywords`, linked `projects` and `utc_start`, but not on `details`.
On a Daylite update, a list sent as an array replaces the whole list, while `{"add": [...], "remove": [...]}` changes only the given entries.

### Evaluated Options
- Synchronize only the edited worker's day: read that worker's calendar and link or unlink that worker's contact on the day's appointments
  - Pros: One calendar read per touched day; another worker's broken calendar cannot fail the write's synchronization; contact links identify workers by reference; `add` and `remove` never resend or drop other links.
  - Cons: Other workers' links are only repaired when their own day is edited; existing assignments gain links one worker at a time; update, move and delete must find the worker of the event they wrote.
- Reconcile the whole day from every planning contact's calendar
  - Pros: Every edit repairs all workers of the day and backfills existing assignments completely.
  - Cons: Reads one calendar per planning contact for every touched day, and one unreadable calendar fails the whole day.
- Keep the worker list in the title instead of contact links
  - Pros: Nothing but the project is linked to the appointment.
  - Cons: Removing a worker means parsing names back out of the title, which breaks on changed nicknames and duplicate names.
- Derive the times and title from the assignments
  - Pros: The appointment shows the planned slot and the workers' names at a glance.
  - Cons: Times depend on every worker's slots, so slot re-allocation and every reorder must be synchronized, and the title has to be rewritten on every change.
- Update the appointment of each of the worker's projects on every synchronization
  - Pros: No owned field has to be read back and compared, so the search needs only references, projects and contacts.
  - Cons: One request per project of the worker's day even when nothing changed.
- Compare the owned fields and update only when one differs
  - Pros: Saves the request for an unchanged appointment.
  - Cons: Every owned field has to be parsed and compared, and a format Daylite returns differently makes every comparison fail anyway.
- Leave duplicates one worker at a time
  - Pros: A synchronization only ever adds or removes the edited worker, also on duplicates.
  - Cons: A duplicate stays until each of its workers was edited.
- Merge duplicates into the lowest Daylite ID in one synchronization
  - Pros: A duplicate disappears with the first synchronization of any worker on it.
  - Cons: Needs a takeover of the other workers' contacts and must delete the duplicates only after that update succeeded.
- Mark app-managed appointments with the keyword `lkr-planner`
  - Pros: Searchable, survives edits in Daylite, and doubles as a visible hint that the app maintains the appointment.
  - Cons: Adds an entry to Daylite's keyword list.
- Mark app-managed appointments with text in `details`
  - Pros: Invisible in the keyword list.
  - Cons: `details` is not searchable, so every appointment of the project would have to be fetched and inspected.
- Report failed days through a backend registry, a Tauri event and a retry command
  - Pros: A later write that heals a worker day also clears it from the alert, and command results stay unchanged.
  - Cons: Failed days are forgotten when the app restarts.
- Run the synchronization inside each command and return its outcome
  - Pros: The frontend learns about a failure from the command it awaited.
  - Cons: Adds Daylite latency to every planner action and changes all five command result types.

## Decision

- After every successful create, update, move and delete, one background task synchronizes the write's touched worker days one after the other: the edited worker on the day the write left and the day it entered, and for a move the target worker first, so the source worker's unlink cannot delete an appointment the target is joining.
- The worker of an updated, moved or deleted event is the employee whose configured primary calendar holds the event, comparing parsed URLs so host case, default ports and path encoding do not matter; an event in no configured calendar is recorded in telemetry and not synchronized.
- A failed CalDAV write and a reorder schedule no synchronization.
- A synchronization reads the day from the worker's primary calendar only, updates the appointment of each project the worker holds and links the worker's contact to it, creating it when missing, and unlinks the contact from appointments of projects the worker no longer holds.
- An appointment whose only linked contact is the worker is deleted instead of unlinked; of several appointments for one project the worker is linked to the lowest Daylite ID and leaves the others like appointments of a project it no longer holds.
- Every appointment is titled `Einsatz <dd.MM.yyyy>` and runs from 08:00 to 16:00 as floating times, with `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, the project as the only linked project and an empty `invitations` list, because Daylite may otherwise invite the requesting user or keep invitees added later.
- App-managed appointments carry the keyword `lkr-planner`; only appointments with that keyword are searched, updated or removed, and updates never send `keywords`.
- Contacts are only ever added or removed one worker at a time, so contacts linked in Daylite or by another device stay linked.
- A process-wide lock per date serializes synchronizations of the same day, so two quick writes cannot both create an appointment.
- A worker day whose calendar cannot be read is not written to Daylite; a failure for one appointment does not stop the others but fails the worker day; every failure is recorded under the telemetry operation `daylite_appointment_sync`.
- The backend keeps the failed worker days in memory and, while holding the set's lock, emits `daylite-appointment-sync-failures` with their sorted ISO dates whenever the set changes.
- The planning grid shows one German error alert naming the failed days and an `Erneut versuchen` button that calls `retry_daylite_appointment_sync`, which synchronizes every failed worker day through the same per-date locks, one at a time.

## Consequences

- Each project day's appointment appears in the project's timeline and with the contact of every worker edited onto it, and nobody is invited.
- Links of workers who are not edited stay as they were, including links left stale by a failed synchronization or a calendar edit outside the app.
- Calendar entries the app did not write gain a link the first time a write touches their worker's day; worker days no write touches are never backfilled.
- Manual edits to an app-managed appointment's title, times, status, type, invitees or linked projects are overwritten on the next synchronization of a worker on it; notes, details, extra keywords and extra contacts are kept.
- An appointment that still links a contact added in Daylite stays after its last worker is unlinked.
- Deleted appointments go to the Daylite trash, which search excludes, so a re-added assignment creates a fresh appointment.
- Failed worker days are lost on restart and heal on the next write that touches them; telemetry still holds the failure.
- Every synchronization sends one update per project of the worker's day, even when nothing changed.
- Concurrent writes from two devices can still create duplicates, and a duplicate disappears only once each of its workers was edited.
