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

- Code: New React component logic for suggestion generation + Rust overdue project query
- Dependencies:
  - BL-022 for project search infrastructure (status filter, timeout, numeric sort)
  - BL-016 for assignment modal where suggestions are displayed

## Note

Overdue project query (previously scoped to BL-022) is implemented here, as it is only consumed by the default suggestions feature. Uses a single Daylite call with `{"category": {"equal": "Überfällig"}}` — no separate status filter needed, as the Daylite API has no multi-value operator for scalar fields and projects in this category are by definition active.
