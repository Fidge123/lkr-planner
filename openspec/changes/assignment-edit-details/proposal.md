## Why

The edit modal can only swap the project behind an assignment.
Everything else a planner knows about a job -- which day it really belongs on, when it actually happens, a title that says more than the project name, a note for the person on site -- has to live outside the planner, so the calendar entry the employee sees in ZEP stays a bare project name in a slot the allocator picked.
Rescheduling already works by dragging a card, but a planner who opens the modal to correct a date has no way to do it there, and nobody can say that a job starts at 07:00 or runs until 18:00.
Marking a job as a fixed appointment is the same story from the other side: the category that decides it lives in Daylite, so a planner who wants to pin a date has to leave the planner to do it.

## What Changes

- The edit modal gains a date field so an assignment can be moved to another day without dragging its card.
- The edit modal gains start and end time fields.
  The entered times apply to the write the planner is making: that assignment is written with them and the day is not re-split around them.
  They are not recorded as manual, so the day's next rearrangement -- another assignment created or deleted, a reorder, a drag -- returns every assignment on it to its share of the 08:00-16:00 window.
  A German hint in the modal says so, because times that quietly disappear later are otherwise indistinguishable from a bug.
- The edit modal gains a checkbox, on by default, that adjusts the day's adjacent assignments to the times just entered: the assignment before ends where this one now starts, the assignment after starts where this one now ends, so the day is left free of gaps and overlaps.
  The fitted neighbours are as transient as the times that caused them.
- The edit modal gains a title field, shown alongside the project field so the custom title and the Daylite project name are both visible.
  An empty title field means no override and the Daylite project name is used, including when it is renamed in Daylite; a title typed into the field replaces it everywhere the assignment is shown.
- The edit modal gains a free-text note that is written into the event DESCRIPTION below the `daylite:` reference line, so it reaches every calendar client reading the event.
- The edit modal gains a category picker holding the project categories Daylite offers, each shown with its name and its color, and saving gives the assignment's Daylite project the picked category.
  The category is the project's, so it applies to every appointment of that project, which a German hint in the modal says out loud.
  A fixed appointment is one whose project carries the category `"Termin FIX geplant"`, so picking that category locks the assignment and picking another releases it.
  Creating categories and editing their names or colors stays in Daylite.
- Title override and note survive every write that rewrites the event: saving the modal, rescheduling by drag, moving to another employee, and slot re-allocation.
  **BREAKING** for `move_assignment`, `update_assignment`, and `create_assignment`: their inputs grow the new fields, and a caller that omits them drops the data.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `assignment-modal-crud`: the modal edits date, start and end time, title, note, and the project's category in addition to the project itself, and its unsaved-changes and validation rules cover the new fields.
- `slot-allocation`: a write can carry times entered by a planner instead of applying the even split, and can fit the day's adjacent assignments to them; those times last only until the day is next re-allocated.
- `assignment-persistence`: assignment events carry a title override and a planner note, and every write path preserves them instead of rebuilding the VEVENT from the project alone.
- `daylite-integration`: the project categories are read as a list carrying each category's name and color, and a project's category can be set.
- `fixed-appointment-protection`: a category set from the planner decides protection right away, instead of the category cached before the write.

## Impact

- Backend: `integrations/calendar/ical.rs` (payload building gains title override and note; parsing gains the new properties), `integrations/calendar/slots.rs` (a write can carry requested times instead of the even split; adjacent adjustment), `caldav/write.rs` (`AssignmentWrite` grows the new fields), `calendar/commands.rs` (command inputs gain the requested times and the adjustment flag), `events/classify.rs` and `events/resolve.rs` (note and title override reach the frontend), `calendar/types.rs` (`CalendarCellEvent`).
- Backend (Daylite): `integrations/daylite/categories.rs` (the category read returns a list), `integrations/daylite/projects.rs` (a project's category can be set, and the cached project follows the write).
- Frontend: `components/assignment-modal.tsx`, `hooks/use-assignment-modal.ts`, `hooks/use-appointment-drag.ts` (drag writes must carry the preserved fields), `services/daylite-categories.ts` (the color lookup is derived from the category list), `app/types.ts`.
- Generated Tauri bindings in `src/generated/tauri.ts` are regenerated.
- No new Rust or npm dependency.
- Saving the modal can now write to Daylite as well as to the calendar, which is the first write this application makes to a project.
- Slot re-allocation gains a second mode: a write can skip the even split for one day, which can leave that day with gaps or overlaps until the next ordinary write tidies it.
