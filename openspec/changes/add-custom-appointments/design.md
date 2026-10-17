# Design

## Context

Every planner-written VEVENT is built by `build_ical_payload` (`src-tauri/src/integrations/calendar/ical.rs`) with a `daylite:<ref>` DESCRIPTION and an `X-LKR-ORDER` property.
`classify_event` (`events/classify.rs`) reads only the DESCRIPTION first line, so an event is either an assignment (`project_ref: Some`) or bare (`None`).
That single `Option` gates everything downstream:

- `resolve_event` maps `None` to `CalendarEventKind::Bare`, which the frontend renders read-only and not draggable.
- `plan_slot_updates` (`slots.rs`) only re-slots events whose `project_ref` is set.
- `is_protected_event` (`protection.rs`) treats a missing reference as protected.
- `AssignmentWrite`, `CreateAssignmentInput`, `UpdateAssignmentInput` and `move_assignment` all take a mandatory `project_ref` and `project_name`.

`patch_event_slot` copies every line it does not rewrite, which is why `X-LKR-ORDER` survives re-slotting (see the archived `appointment-reordering` design).
Full writes (create, update, move) rebuild the payload from scratch.

## Goals / Non-Goals

**Goals:**
- One explicit notion of "what this event links to" that replaces the `Option<String>` everywhere it is currently used as a proxy for "planner-managed".
- Custom appointments reuse every existing write path (create, update, move, reorder, delete, re-slot) instead of getting parallel commands.
- An event the planner did not write never becomes editable by accident.

**Non-Goals:**
- Free start and end times for custom appointments; they take a slot like any assignment.
- Making existing bare events editable or "adopting" them as custom appointments.
- Highlighting by title, title suggestions, or a catalogue of predefined custom titles.
- Next-day quick-add ghosts for custom appointments.
- Multi-day custom appointments.

## Decisions

### Mark custom appointments with an X-property, not a DESCRIPTION prefix
A custom appointment carries `X-LKR-KIND:custom` and has no DESCRIPTION.
Classification order is: a `daylite:` reference makes it a project assignment, otherwise the marker makes it a custom appointment, otherwise it is bare.

Alternative considered: a DESCRIPTION first line such as `lkr:custom`, mirroring the `daylite:` convention.
Rejected because DESCRIPTION is shown to employees in their calendar clients and in ZEP, so a technical marker there is visible noise, and users editing the description in another client could break or fake it.
An X-property is invisible in clients, is already relied on by `X-LKR-ORDER`, and survives `patch_event_slot` without extra handling.
If a client drops it, the event degrades to bare, which is read-only and protected, so the failure is safe.

Record this as ADR 0015 in `docs/adr`.

### Replace `project_ref: Option<String>` with an explicit link enum in the backend
`PendingEvent` gets `link: EventLink` with variants `Project(String)`, `Custom` and `Bare`, derived once in `classify_event`.
`resolve_event`, `plan_slot_updates`, `project_references` and the protection guard match on it, and `CalendarEventKind` gains `Custom`.
`RawVEvent` gains a `custom_marker: bool` filled by `parse_ical_events`, so the guard's `fetch_event_by_href` path sees the marker too.

Alternative considered: keep `project_ref` and add a separate `is_custom` flag.
Rejected because every call site would need to check two fields, and forgetting the second one is exactly the bug class that would make custom appointments silently non-re-slotted or protected.

### One content value on the write commands
`AssignmentWrite`, `CreateAssignmentInput`, `UpdateAssignmentInput` and `move_assignment` replace `project_ref` and `project_name` with `content: AssignmentContent`, a serde-tagged enum:
`{ kind: "project", projectRef, projectName }` or `{ kind: "custom", title }`.
`build_ical_payload` takes the content and writes either the `daylite:` DESCRIPTION or the marker, with the project name or the title as SUMMARY.
The backend trims the custom title and rejects a blank one with a German error, so the modal's disabled save is not the only guard.

Alternatives considered: separate `create_custom_appointment` and siblings, or making `project_ref` optional.
Rejected: parallel commands duplicate the slot, protection, absence-calendar and telemetry wiring for every operation, and an optional `project_ref` cannot carry a title without overloading `project_name`.
The tagged enum also lets `update_assignment` convert between both kinds under the same UID with no extra command.

### Frontend: a new `custom` kind next to `assignment`
`CellEvent.kind` gains `"custom"`, with `projectRef`, `projectStatus` and `categoryColor` always null.
`timetable-cell.tsx` renders custom appointments through the same draggable card as assignments, so dragging, reordering and drop previews need no new code; the card hides the project-only affordances when the kind is `custom`.
Every place that counts or orders "planner cards" (`withDropPreview`, `cell-order.ts`, the drag payload) switches from `kind === "assignment"` to a shared `isPlannerManaged(event)` helper, so the two kinds cannot drift apart.
`highlightKey` stays keyed on `kind === "assignment"`, so custom cards are never highlightable.

### Modal: a type switch above the picker
The modal shows a DaisyUI `join` of two radio buttons, "Projekt" and "Eigener Termin".
In custom mode the project picker and its suggestions are not mounted, and a DaisyUI `input` labeled "Titel" takes their place.
The modal state holds the selected type and the title alongside the selected project, and the dirty check compares both against the opened assignment.
The type switch shares the protected-and-locked disabled condition with the picker.
`ModalSaveAction`'s `create` variant only carries the project for project assignments, and `next-day-quick-add.ts` produces no ghost for a custom create.

## Risks / Trade-offs

- [ZEP or an employee's calendar client drops unknown X-properties on edit] → The event becomes bare and read-only, which is the safe direction. Verify against the real ZEP server that `X-LKR-KIND` survives a round trip, as was done for `X-LKR-ORDER`.
- [Changing the write command signatures breaks every frontend caller at once] → The regenerated bindings make the TypeScript compiler list every call site; there are no external consumers.
- [A long title folds the SUMMARY line] → Folding only excludes events from re-slotting when it affects DTSTART or DTEND, so a long title is harmless; cover it with a test.
- [Planners may use custom appointments for work that should be a Daylite project] → Accepted; the type switch defaults to "Projekt", so the custom path is a deliberate choice.

## Migration Plan

No data migration: existing events have no marker and keep their current classification.
Rollback is a plain revert; custom appointments written in the meantime then show up as read-only bare events, which matches how they would look to any other client.
