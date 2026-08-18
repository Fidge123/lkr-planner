## Why

`prevent-fixed-appointment-modification` locks every assignment whose Daylite project has the category `"Termin FIX geplant"`: the modal's save and delete are disabled and the backend rejects the write.
That is the right default, but it leaves no way to change a fixed appointment from inside the planner at all.
A fixed appointment does occasionally have to move or disappear, and without an in-app path the user has to edit it in another calendar client, where the planner sees nothing and the grid silently drifts out of sync.

The lock should stop accidents, not intent.

## What Changes

- Add an unlock control to the assignment modal's protection notice: while it is off nothing changes, and switching it on re-enables the project picker, the save control and the delete control for that modal session only.
- Reopening the modal drops the unlock again, so an unlocked appointment never stays unlocked.
- Carry the user's decision to the backend as an explicit override on `update_assignment` and `delete_assignment`, which skips the protection check when it is set.
- Keep the guard authoritative about *which* events are protected: it still derives that from the event itself via CalDAV and Daylite, so only the decision to proceed comes from the client.
- No override for `move_assignment`: dragging is not a deliberate enough gesture to carry one, so a protected event still cannot be dragged to another day.

## Capabilities

### Modified Capabilities
- `fixed-appointment-protection`: gains a deliberate-override path that bypasses the guard for a single write.
- `assignment-modal-crud`: gains an unlock control on the protection notice that re-enables the disabled affordances.

## Impact

- `src-tauri/src/integrations/calendar/commands.rs`: `UpdateAssignmentInput` gains an override field, `delete_assignment` gains an override parameter, and both skip `refuse_protected_event` when it is set.
- `src/generated/tauri.ts`: regenerated bindings for both signatures (the exporter runs from the debug app, so this needs one `bun tauri dev` run).
- `src/app/hooks/use-assignment-modal.ts`: unlock state that resets whenever the modal opens, and the flag on both writes.
- `src/app/components/assignment-modal.tsx`: the unlock control inside the existing notice, and the disabled conditions keyed on "protected and not unlocked" rather than "protected".
- `src/app/components/move-reconciliation-dialog.tsx` and `src/app/hooks/use-appointment-drag.ts`: both call the changed commands and need the new argument.
- Tests: `bun test` coverage for locked, unlocked and reopened states.

## Dependencies

Builds on `prevent-fixed-appointment-modification`, which introduces the guard, the notice and the disabled controls this change unlocks.
