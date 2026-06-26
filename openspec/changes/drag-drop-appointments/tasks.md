## 1. Backend: move assignment between calendars

- [ ] 1.1 Write a failing test for `move_assignment_core` covering create-on-target then delete-source success (VCR/recorded CalDAV)
- [ ] 1.2 Write failing tests for the failure paths: target create fails leaves source intact, and a write into an absence calendar is refused
- [ ] 1.3 Implement `move_assignment_core` in `caldav.rs` reusing `create_assignment_core` and `delete_assignment_core`, creating on target first and deleting source only on success
- [ ] 1.4 Add the `move_assignment` Tauri command in `calendar/commands.rs` resolving the target employee's primary calendar from the local store
- [ ] 1.5 Register the command in `lib.rs` and run `cargo test` until green
- [ ] 1.6 Regenerate `generated/tauri.ts` and confirm the `moveAssignment` binding exists

## 2. Frontend: drag state and dispatch hook

- [ ] 2.1 Write failing tests for `use-appointment-drag` (start sets payload, end clears it, edge dwell triggers navigation once and repeats on continued dwell)
- [ ] 2.2 Implement `use-appointment-drag` hook holding the in-flight drag payload and the edge-hover dwell timer
- [ ] 2.3 Implement the drop dispatch: no-op on same employee+date, `updateAssignment` on same employee different date, `moveAssignment` on different employee
- [ ] 2.4 Show a German error and leave the card in place when the target employee has no configured calendar

## 3. Frontend: grid wiring and affordances

- [ ] 3.1 Write failing tests for `TimetableCell`/`TimetableRow` drag behavior: assignment cards are draggable, bare/absence cards are not, plain click still opens the edit modal
- [ ] 3.2 Make assignment cards `draggable` with `onDragStart`/`onDragEnd` wired to the drag hook and a source-dragging indicator
- [ ] 3.3 Make day cells drop targets with `onDragOver`/`onDrop`/`onDragLeave` and a drop-target highlight
- [ ] 3.4 Pass source employee reference and date through `TimetableRow` so the drop handler can decide reschedule vs. move
- [ ] 3.5 Lift a week-navigation callback from `App` through `PlanningGrid`/`page.tsx` and connect it to the hook's edge-hover navigation

## 4. Integration and verification

- [ ] 4.1 Add a grid-level test covering an end-to-end drop that reschedules within an employee and one that reassigns across employees (mocked commands)
- [ ] 4.2 Manually verify drag between days, between employees, and edge-hover week navigation (including multi-week dwell)
- [ ] 4.3 Run `bun lint`, `bun format`, `bun test`, and `cargo test`; fix issues until all green
