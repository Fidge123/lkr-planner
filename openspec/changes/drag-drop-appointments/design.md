## Context

The planning grid renders employees as rows and weekdays as columns (`page.tsx` → `TimetableRow` → `TimetableCell`).
Assignment cards are the only editable events; bare and absence events are read-only.
Today rescheduling goes through `AssignmentModal`, which calls `update_assignment` (in-place PUT to the same href) for date changes.
There is no way to change the employee of an assignment, and the backend has no operation to move a VEVENT between calendars.
Week navigation is owned by `App` via a `weekOffset` state passed down to `PlanningGrid`, and `usePlanningAssignments` already prefetches the previous and next week into a cache.

This change adds direct manipulation: drag an assignment card to another day and/or employee, with edge-hover week navigation so moves can cross week boundaries in one gesture.

## Goals / Non-Goals

**Goals:**
- Drag an assignment card between days (same employee) and between employees.
- Persist a same-calendar date change via the existing `update_assignment`.
- Persist a cross-employee move via a new backend `move_assignment` operation.
- Provide clear source and drop-target affordances during a drag.
- Navigate to the previous/next week when the dragged card dwells over a grid edge, allowing multi-week moves.
- Keep bare and absence events non-draggable and non-editable.

**Non-Goals:**
- Changing an assignment's time-of-day via drag (start/end time are preserved; only date and calendar change).
- Multi-select or dragging more than one card at once.
- Touch-specific gesture support beyond what the native API offers.
- Reordering cards within a single cell.
- Adding a drag-and-drop library dependency.

## Decisions

### Native HTML5 drag-and-drop over a library
Use the browser's native HTML5 DnD API (`draggable`, `onDragStart`, `onDragOver`, `onDrop`, `onDragEnd`) rather than adding `@dnd-kit` or similar.
Rationale: YAGNI — the grid is a simple cell-to-cell move with no sorting or animation requirements, and the native API covers it without a new dependency.
Alternative considered: `@dnd-kit/core` gives nicer pointer/keyboard support and overlays, but adds bundle weight and complexity not justified by the current scope.

### Carry drag payload in a small shared context, not only `dataTransfer`
Drop handlers need the source assignment's `href`, `uid`, `projectRef`, `projectName`, source employee reference, and source date to decide between "no-op", "same-calendar reschedule", and "cross-calendar move".
`dataTransfer` only reliably exposes its data on `drop` (not on `dragover`), so a lightweight React context (or a hook returning shared state) holds the in-flight drag payload for affordance decisions, while `dataTransfer` is still set for correctness.
A new hook `use-appointment-drag` owns the drag payload plus the edge-hover timer.

### Reschedule vs. move dispatch in the drop handler
On drop the handler compares source vs. target:
- Same employee and same date: no-op (return early, no network call).
- Same employee, different date: call `update_assignment(href, uid, targetDate, projectRef, projectName)` — reuses the existing in-place PUT.
- Different employee: call the new `move_assignment(href, targetEmployeeReference, targetDate, projectRef, projectName)`.
After success, call `reloadAssignments()` to refresh the affected weeks.

### Backend `move_assignment` = create-then-delete, target first
CalDAV has no portable atomic cross-collection move, so `move_assignment_core` creates the VEVENT on the target calendar first, and only deletes the source on success.
Rationale: failing safe means never losing the assignment — a failed create leaves the original intact; a failed delete after a successful create leaves a duplicate that the planner can remove, which is preferable to data loss.
It reuses `create_assignment_core` and `delete_assignment_core`, including their existing absence-calendar write guards.
The command resolves the target employee's primary calendar from the local store the same way `create_assignment` does.

### Edge-hover navigation via dwell timer, lifting `weekOffset` callbacks
`App` already owns `weekOffset`; expose `onNavigateWeek(direction)` (or pass `setWeekOffset`) down to the grid.
The drag hook tracks pointer position during `dragover`; when the pointer is within an edge zone (a fixed-width band at the left/right of the scroll container) a timer of the dwell duration starts.
On expiry it triggers navigation in that direction and restarts the timer if the pointer is still in the zone (enabling multi-week jumps).
Leaving the zone or ending the drag clears the timer.
Because adjacent weeks are prefetched, the new week renders without a loading gap and remains a valid drop surface.

## Risks / Trade-offs

- [Create succeeds but delete fails during a cross-employee move, leaving a duplicate] → Surface a German error telling the planner the source copy could not be removed; the duplicate is visible and deletable, and no data is lost.
- [Edge-hover navigation fires unintentionally during normal dragging near edges] → Use a deliberate dwell duration (~700–1000ms) and a narrow edge band so brief passes do not trigger navigation.
- [Native HTML5 DnD interferes with the existing click-to-edit on assignment cards] → Distinguish click from drag via the native drag lifecycle (a completed drag does not fire click); verify the edit modal still opens on a plain click.
- [`dataTransfer` payload unavailable during `dragover` for target-validity styling] → Keep the authoritative payload in the drag hook/context and use `dataTransfer` only as the drop fallback.
- [Stale cache after a move spanning two weeks] → `reloadAssignments` clears the cache and reloads the active week; both source and target weeks are revalidated on next view.

## Migration Plan

No data migration. Ship behind no flag; the feature is additive UI plus one new backend command.
Regenerate `generated/tauri.ts` after adding `move_assignment`.
Rollback is removing the drag handlers and the `move_assignment` command; existing modal-based editing is unaffected.

## Open Questions

- Exact edge-zone width and dwell duration — tune during implementation against real cursor behavior.
- Whether to keep the assignment time-of-day exactly as-is on a cross-employee move (assumed yes; build follows the existing all-day/timed payload behavior of `create_assignment`).
