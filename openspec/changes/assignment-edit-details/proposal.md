## Why

The edit modal can only swap the project behind an assignment.
Everything else a planner knows about a job -- which day it really belongs on, a title that says more than the project name, a note for the person on site, a photo or PDF from the customer -- has to live outside the planner, so the calendar entry the employee sees in ZEP stays a bare project name.
Rescheduling already works by dragging a card, but a planner who opens the modal to correct a date has no way to do it there.

## What Changes

- The edit modal gains a date field so an assignment can be moved to another day without dragging its card.
- The edit modal gains a title field.
  The title defaults to the Daylite project name and only overrides it once the planner types something; an assignment without an override keeps following the project name when it is renamed in Daylite.
- The edit modal gains a free-text note that is written into the event DESCRIPTION below the `daylite:` reference line, so it reaches every calendar client reading the event.
- The edit modal gains file attachments: a planner can attach files to an assignment, see what is attached, open an attachment, and remove one.
  Attachments are stored inside the VEVENT as base64 `ATTACH` properties and are subject to a total size limit per event.
- Title override, note, and attachments survive every write that rewrites the event: saving the modal, rescheduling by drag, moving to another employee, and slot re-allocation.
  **BREAKING** for `move_assignment`, `update_assignment`, and `create_assignment`: their inputs grow the new fields, and a caller that omits them drops the data.
- The create path is unchanged apart from carrying the new fields: a new assignment can be given a title, a note, and attachments in the same dialog.

## Capabilities

### New Capabilities

- `assignment-attachments`: attaching files to an assignment, listing and opening them, removing them, and the size limit and failure behaviour that governs them.

### Modified Capabilities

- `assignment-modal-crud`: the modal edits date, title, and note in addition to the project, and its unsaved-changes and validation rules cover the new fields.
- `assignment-persistence`: assignment events carry a title override, a planner note, and attachments; every write path preserves them instead of rebuilding the VEVENT from the project alone.

## Impact

- Backend: `integrations/calendar/ical.rs` (payload building gains title override, note, attachments, and RFC 5545 line folding; parsing gains the new properties), `caldav/write.rs` (`AssignmentWrite` grows the new fields), `calendar/commands.rs` (command inputs), `events/classify.rs` and `events/resolve.rs` (note and title override reach the frontend), `calendar/types.rs` (`CalendarCellEvent`).
- Frontend: `components/assignment-modal.tsx`, `hooks/use-assignment-modal.ts`, `hooks/use-appointment-drag.ts` (drag writes must carry the preserved fields), `app/types.ts`.
- Generated Tauri bindings in `src/generated/tauri.ts` are regenerated.
- No new Rust or npm dependency: files are read through a browser file input in the webview and shipped to the backend as bytes, and an opened attachment is written to a temp file and handed to the already-installed opener plugin.
- Slot re-allocation is affected indirectly: a folded property that immediately follows `DTSTART`, `DTEND`, or `X-LKR-ORDER` makes an event unpatchable, so the written property order matters.
