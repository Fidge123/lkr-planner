# Design

## Context

Every planner-written VEVENT is built by `build_ical_payload` (`src-tauri/src/integrations/calendar/ical.rs`) with a `daylite:<ref>` DESCRIPTION and an `X-LKR-ORDER` property.
`classify_event` (`events/classify.rs`) reads only the DESCRIPTION first line, so an event is either an assignment (`project_ref: Some`) or bare (`None`).
That single `Option` gates everything downstream:

- `resolve_event` maps `None` to `CalendarEventKind::Bare`, which the frontend renders read-only and not draggable.
- `plan_slot_updates` (`slots.rs`) only re-slots events whose `project_ref` is set.
- `is_protected_event` (`protection.rs`) treats a missing reference as protected.
- `AssignmentWrite`, `CreateAssignmentInput`, `UpdateAssignmentInput` and `move_assignment` all take a mandatory `project_ref` and `project_name`.
- `GhostSuggestion` and `ModalSaveAction` (`src/app/next-day-quick-add.ts`) carry a `projectRef` and `projectName`.

`patch_event_slot` copies every line it does not rewrite, so a DESCRIPTION survives re-slotting unchanged.
Full writes (create, update, move) rebuild the payload from scratch.

## Goals / Non-Goals

**Goals:**
- One explicit notion of "what this event links to" that replaces the `Option<String>` everywhere it is currently used as a proxy for "planner-managed".
- Custom appointments reuse every existing write path (create, update, move, reorder, delete, re-slot, next-day ghost) instead of getting parallel commands.
- An event the planner did not write never becomes editable by accident.

**Non-Goals:**
- Changing the type of an existing entry; a custom appointment stays custom and a project assignment stays a project assignment.
- Free start and end times for custom appointments; they take a slot like any assignment.
- Making existing bare events editable or "adopting" them as custom appointments.
- Highlighting by title, title suggestions, or a catalogue of predefined custom titles.
- Multi-day custom appointments.

## Decisions

### Mark custom appointments with the DESCRIPTION first line `lkr-planner:custom`
A custom appointment's DESCRIPTION is exactly `lkr-planner:custom`.
Classification reads the first line with the normalization already used for `daylite:` (whitespace, BOM and zero-width space trimmed): a `daylite:` reference makes it a project assignment, an exact `lkr-planner:custom` makes it a custom appointment, anything else makes it bare.

The string follows the existing `<scheme>:<value>` convention of `daylite:/v1/projects/42`.
`daylite:` names the system the reference points into; a custom appointment points nowhere, so the scheme names the system that owns the entry instead.
Anyone who sees the line in ZEP or a calendar client can tell which tool wrote it and why it is there, and `custom` reads the same in English code and logs.
An exact match rather than a prefix keeps `lkr-planner:` free for later markers without old events being misread.

Alternatives considered:
- A German string such as `lkr-planner:eigener-termin`, since employees can see it. Rejected because the marker is a machine token, not display text, and the code-is-English convention applies to it.
- An `X-LKR-KIND` property, invisible in clients. Rejected in favor of keeping both planner link types in the one field the classification already reads, and DESCRIPTION is known to survive ZEP round trips.

Since only the first line is read, ZEP or employees can add notes on later lines without breaking the marker.
If a client rewrites the first line, the event degrades to bare, which is read-only and protected, so the failure is safe.

Record this as ADR 0015 in `docs/adr`.

### Replace `project_ref: Option<String>` with an explicit link enum in the backend
`PendingEvent` gets `link: EventLink` with variants `Project(String)`, `Custom` and `Bare`, derived once in `classify_event` from the DESCRIPTION.
`resolve_event`, `plan_slot_updates`, `project_references` and the protection guard match on it, and `CalendarEventKind` gains `Custom`.
The guard parses the same DESCRIPTION it already fetches via `fetch_event_by_href`, so it needs no extra request.

Alternative considered: keep `project_ref` and add a separate `is_custom` flag.
Rejected because every call site would need to check two fields, and forgetting the second one is exactly the bug class that would make custom appointments silently non-re-slotted or protected.

### One content value on the write commands
`AssignmentWrite`, `CreateAssignmentInput`, `UpdateAssignmentInput` and `move_assignment` replace `project_ref` and `project_name` with `content: AssignmentContent`, a serde-tagged enum:
`{ kind: "project", projectRef, projectName }` or `{ kind: "custom", title }`.
`build_ical_payload` takes the content and writes either the `daylite:` or the `lkr-planner:custom` DESCRIPTION, with the project name or the title as SUMMARY.
The backend trims the custom title and rejects a blank one with a German error, so the modal's disabled save is not the only guard.

Alternatives considered: separate `create_custom_appointment` and siblings, or making `project_ref` optional.
Rejected: parallel commands duplicate the slot, protection, absence-calendar and telemetry wiring for every operation, and an optional `project_ref` cannot carry a title without overloading `project_name`.

### Frontend: a new `custom` kind next to `assignment`
`CellEvent.kind` gains `"custom"`, with `projectRef`, `projectStatus` and `categoryColor` always null.
`timetable-cell.tsx` renders custom appointments through the same draggable card as assignments, so dragging, reordering and drop previews need no new code; the card hides the project-only affordances when the kind is `custom`.
Every place that counts or orders "planner cards" (`withDropPreview`, `cell-order.ts`, the drag payload) switches from `kind === "assignment"` to a shared `isPlannerManaged(event)` helper, so the two kinds cannot drift apart.
`highlightKey` stays keyed on `kind === "assignment"`, so custom cards are never highlightable.

### Modal: a type switch above the picker, create mode only
In create mode the modal shows a DaisyUI `join` of two radio buttons, "Projekt" and "Eigener Termin".
In custom mode the project picker and its suggestions are not mounted, and a DaisyUI `input` labeled "Titel" takes their place.
In edit mode the switch is not rendered; the opened entry's kind decides whether the picker or the title field is shown.
The modal state holds the selected type and the title alongside the selected project, and the dirty check compares both against the opened assignment.

### Next-day ghost carries the content, not a project
`GhostSuggestion` and the `create` variant of `ModalSaveAction` carry an `AssignmentContent` instead of `projectRef` and `projectName`.
`nextGhostState` stays unchanged in shape, the ghost card shows the project name or the title, and clicking it calls `create_assignment` with the stored content, so a custom ghost recreates a custom appointment and chaining works for both kinds.

## Risks / Trade-offs

- [Someone types `lkr-planner:custom` into an event's DESCRIPTION by hand in another client] → The event becomes editable in the planner. Accepted: that requires deliberate intent and the planner only edits the employee's own primary calendar.
- [An employee replaces the DESCRIPTION in their calendar client] → The event becomes bare and read-only, which is the safe direction.
- [Changing the write command signatures breaks every frontend caller at once] → The regenerated bindings make the TypeScript compiler list every call site; there are no external consumers.
- [A long title folds the SUMMARY line] → Folding only excludes events from re-slotting when it affects DTSTART or DTEND, so a long title is harmless; cover it with a test.
- [Planners may use custom appointments for work that should be a Daylite project] → Accepted; the type switch defaults to "Projekt", so the custom path is a deliberate choice.

## Migration Plan

No data migration: existing events have no `lkr-planner:custom` line and keep their current classification.
Rollback is a plain revert; custom appointments written in the meantime then show up as read-only bare events, which matches how they would look to any other client.
