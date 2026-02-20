# BL-016: Create/Edit/Delete Assignments in Weekly View (Modal Flow)

## Scope
- Click on an employee/day cell opens an assignment modal.
- Modal default suggestions:
  - First suggestion: most recently assigned project across any employee/day (if available).
  - Below first suggestion: top 4-5 overdue (`Überfällig`) projects.
- Modal includes a text input above suggestions to filter all projects by name.
- Filtered search uses only projects with status `new_status` or `in_progress`.
- While filter text is present, show first 5 matching results and replace default suggestions.
- Support multiple projects per employee/day and multiple employees per project.
- Save changes persistently.
- After assigning a project, show a translucent copy of that project in the next day cell (same employee row) for quick add.

## Acceptance Criteria
- End-to-end flow for assignment CRUD exists.
- Default suggestion order follows: previous assignment first, then overdue projects.
- Filter input replaces default suggestions with first 5 matching results.
- Newly assigned project appears as translucent quick-add suggestion in the next day cell.

## Tests (write first)
- UI tests for modal open/close, Create/Edit/Delete flow.
- UI tests for default suggestion order (previous assignment + overdue list).
- UI tests for filter behavior (replace suggestions, max 5 results).
- UI tests for translucent next-day quick-add behavior.
