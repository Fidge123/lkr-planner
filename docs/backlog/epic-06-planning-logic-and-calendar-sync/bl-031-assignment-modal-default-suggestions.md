# BL-031: Assignment Modal Default Suggestions

## Scope
- Show deterministic default suggestions when assignment modal opens.
- First suggestion must be the most recently assigned project across any employee/day.
- Below the first suggestion show top 4-5 overdue (`Überfällig`) projects.
- Define fallback behavior when recent assignment or overdue projects are unavailable.

## Acceptance Criteria
- Suggestion ordering matches required rule for every modal open.
- Overdue list is capped to 5 entries.
- Empty-state behavior is defined and shown in German.

## Dependencies
- Uses Daylite query capabilities from BL-022.

## Out of Scope
- Free-text filtering behavior (covered by BL-032).

## Tests (write first)
- UI tests for suggestion ordering and count limits.
- Service tests for fallback behavior with missing history/overdue data.
