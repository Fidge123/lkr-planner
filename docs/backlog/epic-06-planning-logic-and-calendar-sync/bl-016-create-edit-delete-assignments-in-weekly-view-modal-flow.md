# BL-016: Assignment Modal CRUD Baseline

## Scope
- Open assignment modal when user clicks an employee/day cell.
- Support assigning and removing projects for the selected employee/day.
- Support editing an existing assignment directly from the same modal.
- Keep flow optimized for day-based planning (no free time input in this baseline).

## Acceptance Criteria
- Modal opens and closes reliably from cell interactions.
- User can create, edit, and delete assignments for one employee/day.
- Persisted state updates immediately in the weekly grid after save.

## Dependencies
- Depends on BL-015 for non-dummy assignment state.

## Out of Scope
- Suggestion ranking and search result replacement behavior.
- Next-day translucent quick-add behavior.

## Tests (write first)
- UI tests for modal open/close from cell interactions.
- UI tests for create/edit/delete actions and persistence confirmation.
- Regression tests for keyboard/cancel/unsaved-change handling.
