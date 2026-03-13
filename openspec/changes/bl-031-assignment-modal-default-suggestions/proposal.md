## Why

The assignment modal needs to show default suggestions when opened. This helps users quickly select frequently used or overdue projects without typing.

## What Changes

- Show deterministic default suggestions when modal opens
- First suggestion: most recently assigned project (across any employee/day)
- Next 4-5 suggestions: overdue projects
- Define fallback behavior when recent assignment or overdue projects unavailable
- Show German empty-state message when no suggestions available

## Capabilities

### New Capabilities
- `assignment-modal-suggestions`: Default suggestions for assignment modal

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New React component logic for suggestion generation
- Dependencies: Uses Daylite query capabilities from BL-022
