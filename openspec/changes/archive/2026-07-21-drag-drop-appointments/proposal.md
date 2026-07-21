## Why

Rescheduling an assignment today requires opening the edit modal, changing the date, and saving, with no way to reassign an employee at all.
Direct drag-and-drop of appointment cards across days and employees makes weekly replanning faster and more intuitive, matching how planners think about the grid.

## What Changes

- Assignment cards (`kind: "assignment"`) become draggable in the planning grid.
- A card can be dropped onto any other day cell of the same employee to reschedule it to that date.
- A card can be dropped onto a cell of a different employee to reassign it, optionally on a different day.
- A move lands on the target day cell and is written with the standard assignment time window, like every other assignment write; positioning before or after existing cards within a cell is out of scope here and handled by the follow-up `appointment-reordering` change.
- While dragging, every droppable cell shows a hover/target indicator and the source cell is visually marked.
- Holding a dragged card over the left or right edge of the grid for a short dwell time triggers navigation to the previous/next week, so a card can be moved across week boundaries in one gesture.
- Bare and absence events stay read-only and are never draggable or droppable as sources.
- A new backend operation moves an assignment between CalDAV calendars (employee reassignment) by creating on the target calendar and deleting from the source.
- When a cross-employee move creates on the target but fails to delete the source, the user is shown a reconciliation dialog (retry delete, keep old and delete new, or keep both) instead of silently leaving a duplicate.

## Capabilities

### New Capabilities
- `appointment-drag-drop`: Dragging assignment cards across day and employee cells, drop-target affordances, edge-hover week navigation during a drag, and reconciliation of a failed cross-calendar move.

### Modified Capabilities
- `assignment-persistence`: Adds a requirement for moving an assignment to a different employee's CalDAV calendar (create on target plus delete from source) with a structured result that distinguishes a full move from a created-but-source-not-deleted state.

## Impact

- Frontend: `timetable-cell.tsx`, `timetable-row.tsx`, `page.tsx` (drag context and week navigation wiring), `app.tsx` (week navigation callback), a new drag-state hook, and a reconciliation dialog component.
- Backend: new `move_assignment` Tauri command and `move_assignment_core` in `caldav.rs` returning a structured move result, regenerated `generated/tauri.ts` bindings.
- New dependency: `@dnd-kit/core` (and `@dnd-kit/utilities`) for pointer-based dragging whose drag state survives the grid re-rendering during mid-drag week navigation. Native HTML5 drag-and-drop cancels a drag when the source element unmounts, which the edge-hover week jump requires; see design.md.
- Reuses existing `update_assignment`, `create_assignment`, and `delete_assignment` CalDAV paths and the assignment reload flow.
