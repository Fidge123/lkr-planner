## Why

Assignments planned in lkr-planner live only in the workers' CalDAV calendars, so a project's Daylite timeline of notes, appointments and emails does not show when and by whom the project was worked on.
Mirroring each planned project day as a Daylite appointment linked to the project and its workers puts that work into the project timeline, without inviting the workers or anyone else.

## What Changes

- Keep one Daylite appointment per project and day on which an edited worker holds an lkr-planner assignment.
- Create that appointment when the first worker is assigned to the project on that day, linked to the project and to the worker's contact, titled `Einsatz <dd.MM.yyyy>`, from 08:00 to 16:00, with type `appointment`, status `confirmed`, show-as `free` and an empty invitee list.
- Link each further worker's contact when they are assigned, and unlink it when their last assignment for that project day is deleted, moved away or changed to another project; workers are never invited.
- Trash the appointment once its last linked planning contact is unlinked.
- After every assignment create, update, move and delete, synchronize only the edited worker's day, as a best-effort follow-up that never fails or rolls back the calendar write.
- Mark app-managed appointments with a Daylite keyword so the app never touches appointments people created by hand.
- Show a German error alert above the planning grid naming the days whose synchronization failed, with an `Erneut versuchen` button that re-runs the synchronization for those days.

## Capabilities

### New Capabilities

- `daylite-appointment-sync`: Mirroring planned project days from the edited workers' assignment calendars into Daylite appointments, covering the one-appointment-per-project-day rule, the appointment's fixed attributes, title and times, the linked worker contacts, the empty invitee list, linking and unlinking on assignment changes, the app-managed marker, failure isolation from calendar writes, and failure reporting with a user-triggered retry.

### Modified Capabilities

None.
The existing `ical-assignment-sync` and `assignment-persistence` requirements keep their behavior; the Daylite synchronization is an additional effect of the same writes and is specified on its own.

## Impact

- `src-tauri/src/integrations/daylite/`: new appointments module with search, create, update, unlink and delete requests against `/v1/appointments`, plus HTTP cassettes for its tests.
- `src-tauri/src/integrations/calendar/`: a synchronization step after each assignment write that reads the edited worker's day and links or unlinks that worker on the day's appointments.
- `src-tauri/src/integrations/calendar/commands.rs`: create, update, move and delete commands trigger the synchronization for the worker days they touched.
- `src-tauri/src/integrations/telemetry/events.rs`: a new operation so synchronization failures are reported.
- A new Tauri command `retry_daylite_appointment_sync` and a new event `daylite-appointment-sync-failures`; existing command signatures stay unchanged and `src/generated/tauri.ts` gains only the new command.
- `src/app/hooks/use-appointment-sync-failures.ts` and `src/app/page.tsx`: the failure alert with its retry button.
- Daylite: appointments carrying the `lkr-planner` keyword appear on linked projects and contacts; the keyword is created in Daylite on first use.
- `docs/adr/0017-daylite-appointments-mirror-assignments.md`: new ADR recording the edited-worker synchronization, the linked contacts, the keyword marker, and the failure reporting with retry.
