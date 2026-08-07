## 1. Confirm the modal actually detects protection

- [ ] 1.1 Run the app against real data and confirm the German notice and the disabled controls appear for an assignment linked to a "Termin FIX geplant" project
- [ ] 1.2 If they do not, establish whether `daylite_list_projects` returns the project at all, and fix the lookup before building the unlock on top of it

## 2. Backend override

- [ ] 2.1 Write failing `cargo test`s for the guard being skipped when the override is set, and still running when it is not
- [ ] 2.2 Add the override field to `UpdateAssignmentInput` and an override parameter to `delete_assignment`, skipping `refuse_protected_event` when set, satisfying the tests
- [ ] 2.3 Confirm `move_assignment` takes no override and still rejects a day change for a protected event

## 3. Generated bindings

- [ ] 3.1 Regenerate `src/generated/tauri.ts` by running the app in debug mode
- [ ] 3.2 Update the other call sites for the changed signatures: the drag path (`use-appointment-drag.ts`) passes no override, the move-reconciliation dialog overrides so a half-finished move can always drop its duplicate

## 4. Modal unlock control

- [ ] 4.1 Write failing `bun test`s: locked state disables picker, save and delete; unlocked state enables all three; reopening resets to locked
- [ ] 4.2 Add the unlock state to `use-assignment-modal.ts`, reset it in the modal's open effect, and pass the override on both writes
- [ ] 4.3 Render the unlock control inside the existing notice in `assignment-modal.tsx` and key the disabled conditions on "protected and not unlocked", satisfying the tests

## 5. Verification

- [ ] 5.1 Run `cargo test` and confirm all new and existing tests pass
- [ ] 5.2 Run `bun test` and confirm all new and existing tests pass
- [ ] 5.3 Run `bun lint` and `bunx tsc --noEmit` and fix any issues
- [ ] 5.4 Manually verify in the running app: a fixed appointment is locked on open, the unlock control re-enables editing and deleting, both writes go through, and reopening the modal locks it again
