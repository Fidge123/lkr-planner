## Context

The planning grid renders employees as rows and weekdays as columns (`page.tsx` → `TimetableRow` → `TimetableCell`).
Assignment cards are the only editable events; bare and absence events are read-only.
Today rescheduling goes through `AssignmentModal`, which calls `update_assignment` (in-place PUT to the same href) for date changes.
There is no way to change the employee of an assignment, and the backend has no operation to move a VEVENT between calendars.
Week navigation is owned by `App` via a `weekOffset` state passed down to `PlanningGrid`, and `usePlanningAssignments` already prefetches the previous and next week into a cache.

This change adds direct manipulation: drag an assignment card to another day and/or employee, with edge-hover week navigation so moves can cross week boundaries in one gesture.
Within-cell positioning (drop before/after a specific card) and intra-day reordering are deliberately deferred to the follow-up `appointment-reordering` change, which introduces a persisted order index and coordinates with BL-034 slot allocation.

## Goals / Non-Goals

**Goals:**
- Drag an assignment card between days (same employee) and between employees.
- Persist a same-calendar date change via the existing `update_assignment`.
- Persist a cross-employee move via a new backend `move_assignment` operation.
- Provide clear source and drop-target affordances during a drag.
- Navigate to the previous/next week when the dragged card dwells over a grid edge, allowing multi-week moves in one continuous drag.
- Recover gracefully when a cross-employee move creates the target but cannot delete the source.
- Keep bare and absence events non-draggable and non-editable.

**Non-Goals:**
- Changing an assignment's time-of-day via drag; every write path normalizes to the standard 08:00 to 16:00 assignment window, so only date and calendar change.
- Positioning a card before/after existing cards within a cell, or reordering within a day (handled by `appointment-reordering`).
- Multi-select or dragging more than one card at once.
- Reordering cards within a single cell.
- Keyboard-driven dragging (no dnd-kit KeyboardSensor); dragging is pointer-only, keyboard users reschedule via the edit modal.

## Decisions

### dnd-kit over native HTML5 drag-and-drop
Use `@dnd-kit/core` (pointer sensors) rather than the native HTML5 drag-and-drop API.
The deciding constraint is edge-hover week navigation: dwelling at an edge calls `setWeekOffset`, which unmounts the entire week `<tbody>` — including the dragged source card — and mounts a fresh one.
Native HTML5 DnD cancels the drag the moment the source element leaves the DOM, so the gesture "dwell at edge, jump week, drop in the new week" is impossible on the native API.
dnd-kit is intentionally not built on native DnD: it drives the drag from its own pointer state and renders the dragged preview in a `DragOverlay` React portal mounted at the document root, so the drag survives the grid reconciling underneath it.
Bonus: dnd-kit activation constraints (pointer distance/delay) cleanly separate a click-to-edit from a drag, and pointer coordinates from drag-move events make edge-zone detection straightforward.

Alternatives considered:
- Native HTML5 DnD (the original plan): zero dependency, but fatally cancels on source unmount; rejected.
- Atlassian Pragmatic drag-and-drop: lighter than dnd-kit but built on the native HTML5 DnD API, so it inherits the same source-unmount cancellation; rejected for this feature.
- Hand-portaling the dragged card to keep it mounted across navigation: effectively reimplementing dnd-kit's DragOverlay by hand, fragile; rejected.

### Carry drag payload in dnd-kit's drag context
The drop handler needs the source assignment's `href`, `uid`, `projectRef`, `projectName`, source employee reference, and source date to choose between "no-op", "same-calendar reschedule", and "cross-calendar move".
This rides on dnd-kit's draggable `data` and the active-drag context rather than a separate store.
A thin `use-appointment-drag` hook wraps the dnd-kit context and owns the edge-hover dwell timer.

### Reschedule vs. move dispatch in the drop handler
On drop the handler compares source vs. target droppable:
- Same employee and same date: no-op (return early, no network call).
- Same employee, different date: call `update_assignment(href, uid, targetDate, projectRef, projectName)` — reuses the existing in-place PUT.
- Different employee: call the new `move_assignment(href, targetEmployeeReference, targetDate, projectRef, projectName)`.
After success, call `reloadAssignments()` to refresh the affected weeks.
The write rebuilds the VEVENT with the standard assignment time window, exactly like the modal edit path.

### Backend `move_assignment` = create-then-delete, target first, structured result
CalDAV has no portable atomic cross-collection move, so `move_assignment_core` creates the VEVENT on the target calendar first, and only then deletes the source.
It returns a structured result rather than a bare href so the frontend can react to a partial move:
- `Moved { new_href }`: target created and source deleted.
- `SourceDeleteFailed { new_href, source_href }`: target created but the source delete failed; the assignment now exists twice.
A failed target create returns an error and leaves the source untouched (no data loss).
It reuses `create_assignment_core` and `delete_assignment_core`, including their existing absence-calendar write guards, and resolves the target employee's primary calendar from the local store the same way `create_assignment` does.

### Reconciliation dialog for a partial move
Silently keeping a duplicate is not acceptable.
When `move_assignment` returns `SourceDeleteFailed`, the UI opens a reconciliation dialog with three German options:
- Retry deleting the source (`delete_assignment(source_href)`).
- Keep the old one and delete the new copy (`delete_assignment(new_href)`).
- Keep both (dismiss; the planner resolves it manually).
After any choice the grid reloads. The dialog is modal and blocks until the planner decides.

### Edge-hover navigation via dwell timer, lifting `weekOffset` callbacks
`App` already owns `weekOffset`; expose `onNavigateWeek(direction)` down to the grid.
The drag hook tracks pointer position during drag-move; when the pointer enters an edge zone (a fixed-width band at the left/right of the scroll container) a dwell timer starts.
On expiry it triggers navigation in that direction and restarts the timer if the pointer is still in the zone (enabling multi-week jumps).
Leaving the zone or ending the drag clears the timer.
Because adjacent weeks are prefetched and the DragOverlay is portal-mounted, the new week renders without a loading gap and remains a valid drop surface mid-drag.

## Risks / Trade-offs

- [Create succeeds but source delete fails during a cross-employee move] → `move_assignment` reports `SourceDeleteFailed` and the UI forces a reconciliation dialog; no duplicate is ever left silently and no data is lost.
- [Edge-hover navigation fires unintentionally during normal dragging near edges] → Use a deliberate dwell duration (~700–1000ms) and a narrow edge band so brief passes do not trigger navigation.
- [dnd-kit pointer dragging conflicts with click-to-edit on assignment cards] → Configure an activation constraint (small pointer distance or short delay) so a plain click still opens the edit modal; verify in tests.
- [New dependency weight] → ~6KB core for `@dnd-kit/core`; justified because the edge-hover week-jump feature is impossible on the dependency-free native API.
- [Stale cache after a move spanning two weeks] → `reloadAssignments` clears the cache and reloads the active week; both source and target weeks are revalidated on next view.

## Migration Plan

No data migration. Ship behind no flag; the feature is additive UI plus one new backend command.
Add `@dnd-kit/core` (and `@dnd-kit/utilities` if needed) via Bun, and regenerate `generated/tauri.ts` after adding `move_assignment`.
Rollback is removing the drag handlers, the dnd-kit dependency, and the `move_assignment` command; existing modal-based editing is unaffected.

## Open Questions

- Exact edge-zone width and dwell duration — tune during implementation against real cursor behavior.
- Activation-constraint tuning (distance vs. delay) so click-to-edit and drag feel natural.
