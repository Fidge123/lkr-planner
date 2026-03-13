## Why

The assignment modal needs live filtering as the user types. This allows quick project finding without navigating away from the modal.

## What Changes

- Add text input field above default suggestions
- While filter text is present, replace default suggestions with filtered results
- Show first 5 matching projects from `new_status` and `in_progress` states
- Support keyboard selection (arrow keys + enter) in filtered list
- Restore default suggestions when input is cleared

## Capabilities

### New Capabilities
- `assignment-modal-filter`: Live text filter for assignment modal

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New React component for filter input and filtered list
- Dependencies: Uses Daylite query capabilities from BL-022
