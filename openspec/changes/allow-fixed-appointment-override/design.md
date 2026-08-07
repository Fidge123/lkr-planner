## Context

`prevent-fixed-appointment-modification` adds `refuse_protected_event` (`src-tauri/src/integrations/calendar/protection.rs`), called by `update_assignment` and `delete_assignment` before any CalDAV write.
It reads the event by `href`, parses the `daylite:/<path>` reference from its DESCRIPTION and looks up that project's category, so protection is derived from the event and never from a caller-supplied value.
The assignment modal renders a German notice and disables save and delete when the assignment's `projectCategory` carries the category (`src/app/hooks/use-assignment-modal.ts`).
`move_assignment` is guarded separately by `refuse_protected_day_change`, which only rejects a move that changes the event's day.

An earlier attempt at this override was implemented and reverted (commit `7ae39e8`, reverted by `1b3f4bc`) because it landed without a proposal.
The reverted work is a usable reference for the shape of the change, not a decision that has been made.

During that attempt the unlock control did not appear in the running app.
The likely cause was the lookup it sat on: the modal resolved the category from a frontend cache of all Daylite projects, which could miss a project the server paginated away.
That lookup has since been replaced by the `project_category` field the backend resolves per event, so the detection should be reliable; confirm the notice appears before building the unlock on top of it.

## Goals / Non-Goals

**Goals:**
- Give the user an in-app path to change or remove a fixed appointment deliberately.
- Keep the accident case fully protected: every write that does not carry the override is still checked.
- Keep the override per write and per modal session, so it cannot linger.

**Non-Goals:**
- No override for drag-and-drop. A drag is a fast gesture with no room for a deliberate confirmation, so a protected event stays undraggable across days.
- No persisted or per-project unlock, and no "unlock for the rest of the session".
- No audit trail of overridden writes (add later if the user wants to see who unlocked what).

## Decisions

### The override is a write parameter, not a protection input
`update_assignment` and `delete_assignment` take an explicit override flag and skip the guard entirely when it is set.
The guard keeps deriving protection from the event itself, so the client still cannot influence *which* events are protected, only whether the user chose to proceed with this one write.
Alternative considered: have the frontend pass the project category it believes applies.
Rejected for the same reason the guard exists: a client-supplied category would let a modified frontend unprotect anything silently, while an override is visible in the command's signature and requires the user's action.

### Skipping the guard also skips its fetch
An overridden write performs no CalDAV GET and no Daylite lookup, so the override path is cheaper than the guarded one rather than more expensive.

### Unlock lives in the notice, not next to the buttons
The unlock control belongs inside the existing warning block, so the explanation and the way past it read as one unit.
Alternative considered: a separate confirmation dialog on save, like the delete confirmation.
Rejected because it moves the decision away from the explanation and adds a second modal layer to a flow that already has two.

### Reopening the modal clears the unlock
The unlock is component state reset by the modal's open effect, matching how the delete-confirm and unsaved-changes state already reset.
This keeps "unlocked" tied to one visible, deliberate interaction.

## Risks / Trade-offs

- [An override makes the guard bypassable by construction] → Accepted: the guard's purpose is to prevent accidents, and a user who ticks an unlock control in a warning block is not having one. Every unattended path (drag, re-slotting, stale UI) still goes through the check.
- [Users may learn to tick the unlock reflexively] → Mitigate by resetting it on every open, so the cost is paid each time rather than once.
- [The modal's protection detection may not fire in the real app] → Confirm the notice appears for a real fixed appointment before building the unlock on top of it; if it does not, the `project_category` the backend resolves has to be fixed first or the unlock control will never be reachable.

## Open Questions

- Should an overridden write be logged, so it is possible to see later that a fixed appointment was changed on purpose?
