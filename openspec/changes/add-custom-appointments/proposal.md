# Proposal

## Why

Every entry the planner writes today must reference a Daylite project, so work that has no project (workshop day, training, vehicle service, internal meeting) cannot be planned in the grid at all.
Planners fall back to creating such entries in another calendar client, where they show up as read-only bare events that cannot be edited, moved, reordered or deleted from the planner and do not take part in slot allocation.

## What Changes

- Introduce custom appointments: planner-managed calendar entries with a free-text title and no Daylite project.
- Mark a custom appointment with the DESCRIPTION first line `lkr-planner:custom`, so it can be told apart from a project assignment (`daylite:` reference) and from a read-only bare event written by someone else.
- Add an appointment type switch to the assignment modal ("Projekt" / "Eigener Termin") in create mode; in custom mode a title field replaces the project picker.
- An existing entry keeps its type: the switch is not shown in edit mode, so a custom appointment cannot become a project assignment or vice versa.
- Let custom appointments be edited, deleted, dragged to another day or employee, reordered within a day and re-slotted, exactly like project assignments.
- Show the next-day quick-add ghost after creating a custom appointment too, so a multi-day training can be chained day by day.
- Treat custom appointments as never protected: the fixed-appointment guard only applies to Daylite projects with category `"Termin FIX geplant"` and to bare events.
- Render custom appointments as editable cards that carry no project status, no category strip color and no highlight toggle.
- Replace the separate `projectRef` / `projectName` arguments of `create_assignment`, `update_assignment` and `move_assignment` with one content value that is either a project or a custom title.
- Bare events written by other clients stay read-only and protected; this change does not make them editable.

## Capabilities

### New Capabilities
- `custom-appointments`: identifying, creating, editing, deleting and rendering planner-managed appointments that have a free-text title instead of a Daylite project.

### Modified Capabilities
- `assignment-persistence`: event display distinguishes three planner tiers instead of two, and moving between calendars carries either a project or a custom title.
- `fixed-appointment-protection`: an event without a Daylite reference is only protected when it is not a custom appointment.
- `slot-allocation`: custom appointments are re-slotted alongside project assignments.
- `appointment-drag-drop`: custom appointment cards are draggable.
- `assignment-modal-crud`: the modal saves either a project assignment or a custom appointment.
- `next-day-quick-add`: the ghost also follows a created custom appointment and recreates it with the same title.

## Impact

- `src-tauri/src/integrations/calendar/ical.rs`: payload builder writes either the `daylite:` or the `lkr-planner:custom` DESCRIPTION.
- `src-tauri/src/integrations/calendar/events/classify.rs` and `resolve.rs`: classify the `lkr-planner:custom` first line into a new `custom` event kind.
- `src-tauri/src/integrations/calendar/slots.rs`: the re-slot filter includes custom appointments.
- `src-tauri/src/integrations/calendar/protection.rs`: custom appointments are never protected.
- `src-tauri/src/integrations/calendar/commands.rs`, `caldav/write.rs`, `types.rs`: content enum on create, update and move; `CalendarEventKind::Custom`.
- `src/generated/tauri.ts`: regenerated bindings.
- `src/app/types.ts`, `src/app/components/timetable-cell.tsx`, `src/app/components/assignment-modal.tsx`, `src/app/hooks/use-assignment-modal.ts`, `src/app/hooks/use-appointment-drag.ts`, `src/app/components/move-reconciliation-dialog.tsx`, `src/app/next-day-quick-add.ts`: new kind, card rendering, type switch and title field, ghosts carrying either a project or a title, content argument on all writes.
- `docs/adr`: new ADR for the custom appointment marker.
- No new dependencies and no Daylite or Planradar API changes.
