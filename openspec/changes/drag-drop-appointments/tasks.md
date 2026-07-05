## 1. Backend: move assignment between calendars

- [x] 1.1 Write a failing test for `move_assignment_core` covering create-on-target then delete-source returning a full-move result (VCR/recorded CalDAV)
- [x] 1.2 Write a failing test for the partial-move path: target create succeeds, source delete fails, result reports both new and source hrefs and the source is left intact
- [x] 1.3 Write failing tests for the reject paths: target create fails leaves source intact, and a write into an absence calendar is refused
- [x] 1.4 Implement `move_assignment_core` in `caldav.rs` reusing `create_assignment_core` and `delete_assignment_core`, creating on target first, returning a structured result (full move vs. source-delete-failed)
- [x] 1.5 Add the `move_assignment` Tauri command in `calendar/commands.rs` resolving the target employee's primary calendar from the local store
- [x] 1.6 Register the command in `lib.rs` and run `cargo test` until green
- [x] 1.7 Regenerate `generated/tauri.ts` and confirm the `moveAssignment` binding and result type exist

## 2. Frontend: dnd-kit setup and drag state

- [x] 2.1 Add `@dnd-kit/core` (and `@dnd-kit/utilities` if needed) via Bun and wrap the grid in a `DndContext` with a pointer sensor and an activation constraint that preserves click-to-edit
- [x] 2.2 Write failing tests for `use-appointment-drag` (start exposes payload, end clears it, edge dwell triggers navigation once and repeats on continued dwell)
- [x] 2.3 Implement `use-appointment-drag` wrapping the dnd-kit context and owning the edge-hover dwell timer
- [x] 2.4 Implement the drop dispatch: no-op on same employee+date, `updateAssignment` on same employee different date, `moveAssignment` on different employee, preserving time-of-day
- [x] 2.5 Show a German error and leave the card in place when the target employee has no configured calendar

## 3. Frontend: grid wiring and affordances

- [x] 3.1 Write failing tests for `TimetableCell`/`TimetableRow` drag behavior: assignment cards are draggable, bare/absence cards are not, plain click still opens the edit modal
- [x] 3.2 Make assignment cards draggable via dnd-kit with a source-dragging indicator and a `DragOverlay` preview rendered through a portal so it survives week navigation
- [x] 3.3 Make day cells droppable with a drop-target highlight
- [x] 3.4 Pass source employee reference and date through `TimetableRow` so the drop handler can decide reschedule vs. move
- [x] 3.5 Lift a week-navigation callback from `App` through `PlanningGrid`/`page.tsx` and connect it to the hook's edge-hover navigation

## 4. Frontend: reconciliation dialog

- [x] 4.1 Write failing tests for the reconciliation dialog: it opens on a partial-move result and offers retry-delete-source, keep-old-delete-new, and keep-both
- [x] 4.2 Implement the reconciliation dialog component with the three German options, wiring retry/keep-old to `deleteAssignment` and keep-both to dismissal
- [x] 4.3 Trigger the dialog from the drop dispatch when `moveAssignment` returns a partial-move result and reload the grid after any choice

## 5. Integration and verification

- [x] 5.1 Add a grid-level test covering an end-to-end drop that reschedules within an employee and one that reassigns across employees (mocked commands)
- [x] 5.2 Manually verify drag between days, between employees, edge-hover week navigation (including multi-week dwell), and the reconciliation dialog on a simulated source-delete failure
- [x] 5.3 Run `bun lint`, `bun format`, `bun test`, and `cargo test`; fix issues until all green
