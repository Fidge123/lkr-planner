# BL-033: Next-Day Translucent Quick Add

## Scope
- After assigning a project to a day, show a translucent copy in the next day cell for same employee.
- Allow one-click add from the translucent item.
- Ensure translucent items are clearly distinguishable from persisted assignments.
- Remove/update translucent suggestions when source assignment changes.

## Acceptance Criteria
- Newly saved assignment creates one next-day translucent suggestion.
- Quick-add creates a real assignment for next day.
- Visual state clearly separates suggestion vs persisted assignment.

## Dependencies
- Depends on BL-016 assignment modal CRUD baseline.

## Out of Scope
- Multi-day auto-propagation beyond next day.

## Tests (write first)
- UI tests for translucent rendering after assignment save.
- UI tests for one-click conversion to persisted assignment.
- UI tests for cleanup when source assignment is deleted/changed.
