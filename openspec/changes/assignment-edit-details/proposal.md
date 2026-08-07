## Why

The edit modal can only swap the project behind an assignment.
Everything else a planner knows about a job -- which day it really belongs on, when it actually happens, a title that says more than the project name, a note for the person on site -- has to live outside the planner, so the calendar entry the employee sees in ZEP stays a bare project name in a slot the allocator picked.
Rescheduling already works by dragging a card, but a planner who opens the modal to correct a date has no way to do it there, and nobody can say that a job starts at 07:00 or runs until 18:00.

## What Changes

- The edit modal gains a date field so an assignment can be moved to another day without dragging its card.
- The edit modal gains start and end time fields.
  An assignment whose times were set by hand keeps them: it is excluded from automatic slot allocation, the same way an assignment that cannot be rewritten safely is excluded today, and it takes no share of the 08:00-16:00 window.
  Clearing both fields hands the assignment back to automatic allocation.
- The edit modal gains a checkbox, on by default, that adjusts the day's adjacent assignments to the times just set: the assignment before ends where this one now starts, the assignment after starts where this one now ends, so the day stays free of gaps and overlaps.
  Those neighbours become manually timed too, because their times no longer follow the even split.
- The edit modal gains a title field.
  The title defaults to the Daylite project name and only overrides it once the planner types something; an assignment without an override keeps following the project name when it is renamed in Daylite.
- The edit modal gains a free-text note that is written into the event DESCRIPTION below the `daylite:` reference line, so it reaches every calendar client reading the event.
- Title override, note, and manual times survive every write that rewrites the event: saving the modal, rescheduling by drag, moving to another employee, and slot re-allocation.
  **BREAKING** for `move_assignment`, `update_assignment`, and `create_assignment`: their inputs grow the new fields, and a caller that omits them drops the data.
- **BREAKING** for `appointment-drag-drop`: a drag no longer always writes the standard time window, because a manually timed assignment keeps its times across a drag.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `assignment-modal-crud`: the modal edits date, start and end time, title, and note in addition to the project, and its unsaved-changes and validation rules cover the new fields.
- `slot-allocation`: an assignment can be pinned to times set by hand, which excludes it from allocation, and a write can adjust the day's adjacent assignments to fit the times just set.
- `assignment-persistence`: assignment events carry a title override, a planner note, and a marker that their times were set by hand; every write path preserves them instead of rebuilding the VEVENT from the project alone.
- `appointment-drag-drop`: a dragged assignment that carries manual times keeps them instead of being written with the standard window.

## Impact

- Backend: `integrations/calendar/ical.rs` (payload building gains title override, note, manual times and their marker; parsing gains the new properties), `integrations/calendar/slots.rs` (manually timed assignments leave the allocation; adjacent adjustment), `caldav/write.rs` (`AssignmentWrite` grows the new fields), `calendar/commands.rs` (command inputs), `events/classify.rs` and `events/resolve.rs` (note and title override reach the frontend), `calendar/types.rs` (`CalendarCellEvent`).
- Frontend: `components/assignment-modal.tsx`, `hooks/use-assignment-modal.ts`, `hooks/use-appointment-drag.ts` (drag writes must carry the preserved fields), `app/types.ts`.
- Generated Tauri bindings in `src/generated/tauri.ts` are regenerated.
- No new Rust or npm dependency.
- Slot re-allocation is affected directly: the set of events taking part in an allocation shrinks, and the day's window may now be shared unevenly or left with gaps.
