## Context

After creating an assignment, users often want to continue assigning the same project to subsequent days.
A single translucent ghost on the next visible day offers a one-click way to continue.
The ghost is a pure frontend overlay.
It never comes back from CalDAV and does not survive an app restart, but it does survive the table reload that a save triggers, so it can drive the next create.

## Goals / Non-Goals

**Goals:**
- Show one translucent ghost on the next visible day after a create.
- Allow one-click conversion to a persisted assignment with day-by-day chaining.
- Visually distinguish the ghost from real assignments.
- Clear the ghost on any delete and on week navigation.

**Non-Goals:**
- Showing ghosts on more than one day at a time.
- Skipping ahead to the next free day when the target day is occupied.
- Persisting the ghost across app restarts.
- Triggering a ghost from edits or deletes.

## Decisions

### Single ghost owned by the row
**Decision**: The ghost is a single optional record `{ date, project } | null` owned by `TimetableRow`.
A ghost is always the same employee as its source, so it never needs to leave the row.
There is at most one ghost at any time.
This removes the earlier "employee plus day mapping" and any source-tracking bookkeeping.

### Create-only trigger
**Decision**: Only `createAssignment` sets a ghost.
Edits and deletes never set one.
`TimetableRow` already knows create versus edit from `modalState.assignment` being `null`.
The `onSave` callback carries the created project (reference plus name) so the row can render the ghost and re-create on click.
Edit and delete paths call `onSave` without that payload.

### Target day and suppression at render time
**Decision**: The target day is the next entry in `weekDays` after the created day.
If the created day is the last visible day, there is no target and no ghost.
Suppression is evaluated at render against live props, not latched at save time.
Each cell shows the ghost only when `cell.date === ghost.date` and the cell holds no events.
Because the table reloads after a save, an occupied day automatically hides the ghost with no extra bookkeeping.
"No events" includes assignments, bare events, and absences, so the ghost never lands on an out-of-office day.

### Rendering via a separate prop
**Decision**: `TimetableCell` gains an optional `suggestion` prop.
It renders between the events and the `+` button with reduced opacity (~50%) and a dashed border.
This keeps the non-persisted ghost out of the `CellEvent` union used for real events.

### Chaining falls out of the create path
**Decision**: Clicking a ghost reuses the create path: it calls `createAssignment`, then sets the next ghost one visible day further and reloads the table.
Because a click is itself a create, chaining is automatic and honors suppression and the last-visible-day boundary.

### Clearing
**Decision**: Any delete clears the ghost.
Week navigation clears it via an effect that watches the week.
The row persists across week changes because it is keyed by employee, so the effect is required.
Cancelling a modal is neither a create nor a delete and leaves the ghost untouched.

## Risks / Trade-offs

- **Risk**: User confusion between a ghost and a real assignment.
  - **Mitigation**: Clear visual distinction with opacity and dashed border.
- **Risk**: A stale ghost lingers after the underlying data changes.
  - **Mitigation**: Render-time suppression against live cell events, plus clearing on delete and week navigation.
