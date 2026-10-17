## Why

Assignments planned in lkr-planner live only in the workers' CalDAV calendars, so a project's Daylite timeline of notes, appointments and emails does not show when and by whom the project was worked on.
Mirroring each planned project day as a Daylite appointment linked to the project puts that work into the project timeline, without inviting the workers or anyone else.

## What Changes

- Keep exactly one Daylite appointment per project and day for which at least one worker has an lkr-planner assignment.
- Create that appointment when the first worker is assigned to the project on that day, linked to the project, with type `appointment`, status `confirmed`, show-as `free` and an empty invitee list.
- Name the appointment after the project and list the assigned workers' names in its title; workers are never added as invitees or linked contacts.
- Take the appointment's start and end from the assignment events in the calendar: the earliest start and the latest end of that project's assignments on that day.
- Rewrite the title and times when a further worker is assigned, when a worker is removed, or when slot re-allocation shifts the assignment times.
- Remove a worker's name from the title when their assignment is deleted or moved away, and trash the appointment once nobody is assigned to the project on that day.
- Run this synchronization after every assignment create, update, move, reorder and delete, as a best-effort follow-up that never fails or rolls back the calendar write.
- Mark app-managed appointments with a Daylite keyword so the app never touches appointments people created by hand.
- Show a German error alert above the planning grid naming the days whose synchronization failed, with an `Erneut versuchen` button that re-runs the synchronization for those days.

## Capabilities

### New Capabilities

- `daylite-appointment-sync`: Mirroring planned project days from the workers' assignment calendars into Daylite appointments, covering the one-appointment-per-project-day rule, the appointment's fixed attributes, title and time derivation, the empty invitee list, update and removal on assignment changes, the app-managed marker, failure isolation from calendar writes, and failure reporting with a user-triggered retry.

### Modified Capabilities

None.
The existing `ical-assignment-sync` and `assignment-persistence` requirements keep their behavior; the Daylite synchronization is an additional effect of the same writes and is specified on its own.

## Impact

- `src-tauri/src/integrations/daylite/`: new appointments module with search, create, edit and delete requests against `/v1/appointments`, plus HTTP cassettes for its tests.
- `src-tauri/src/integrations/calendar/`: a reconciliation step after each assignment write that reads the affected day from every configured primary calendar and computes the desired appointment per project.
- `src-tauri/src/integrations/calendar/commands.rs`: create, update, move, reorder and delete commands trigger the reconciliation for the days and projects they touched.
- `src-tauri/src/integrations/telemetry/events.rs`: a new operation so synchronization failures are reported.
- A new Tauri command `retry_daylite_appointment_sync` and a new event `daylite-appointment-sync-failures`; existing command signatures stay unchanged and `src/generated/tauri.ts` gains only the new command.
- `src/app/hooks/use-appointment-sync-failures.ts` and `src/app/page.tsx`: the failure alert with its retry button.
- Daylite: appointments carrying the `lkr-planner` keyword appear on linked projects; the keyword is created in Daylite on first use.
- `docs/adr/0015-daylite-appointments-mirror-assignments.md`: new ADR recording the desired-state reconciliation, the keyword marker, and the failure reporting with retry.
