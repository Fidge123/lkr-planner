# BL-032: Assignment Modal Live Project Filter

## Scope
- Add text input above default suggestions to filter projects by name.
- While filter text is present, replace default suggestions with filtered result list.
- Show first 5 matching projects from `new_status` and `in_progress` states.
- Support quick keyboard selection in filtered list.

## Acceptance Criteria
- Typing in filter input replaces default suggestion block.
- Filtered list contains max 5 entries.
- Clearing input restores default suggestion block.

## Dependencies
- Uses Daylite query capabilities from BL-022.

## Out of Scope
- Persisting personal search history.

## Tests (write first)
- UI tests for replace/restore behavior when input changes.
- Service tests for max-5 and status-filter guarantees.
- UI tests for keyboard selection behavior.
