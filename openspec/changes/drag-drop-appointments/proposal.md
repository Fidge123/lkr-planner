## Why

Rescheduling an assignment today requires opening the edit modal, changing the date, and saving, with no way to reassign an employee at all.
Direct drag-and-drop of appointment cards across days and employees makes weekly replanning faster and more intuitive, matching how planners think about the grid.

## What Changes

- Assignment cards (`kind: "assignment"`) become draggable in the planning grid.
- A card can be dropped onto any other day cell of the same employee to reschedule it to that date.
- A card can be dropped onto a cell of a different employee to reassign it, optionally on a different day at the same time.
- While dragging, every droppable cell shows a hover/target indicator and the source cell is visually marked.
- Holding a dragged card over the left or right edge of the grid for a short dwell time triggers navigation to the previous/next week, so a card can be moved across week boundaries in one gesture.
- Bare and absence events stay read-only and are never draggable or droppable as sources.
- A new backend operation moves an assignment between CalDAV calendars (employee reassignment) since the existing in-place update only handles same-calendar date changes.

## Capabilities

### New Capabilities
- `appointment-drag-drop`: Dragging assignment cards across day and employee cells, drop-target affordances, and edge-hover week navigation during a drag.

### Modified Capabilities
- `assignment-persistence`: Adds a requirement for moving an assignment to a different employee's CalDAV calendar (create on target plus delete from source), distinct from the existing same-calendar in-place update.

## Impact

- Frontend: `timetable-cell.tsx`, `timetable-row.tsx`, `page.tsx` (drag context and week navigation wiring), `app.tsx` (week navigation callback), plus a new drag-state hook.
- Backend: new `move_assignment` Tauri command and `move_assignment_core` in `caldav.rs`, regenerated `generated/tauri.ts` bindings.
- No new third-party dependency: uses the native HTML5 drag-and-drop API.
- Reuses existing `update_assignment`, `create_assignment`, and `delete_assignment` CalDAV paths and the assignment reload flow.
