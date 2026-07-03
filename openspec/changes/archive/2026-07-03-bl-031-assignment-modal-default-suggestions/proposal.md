## Why

The assignment modal needs to show default suggestions when opened. This helps users quickly select frequently used or overdue projects without typing.

## What Changes

- Show deterministic default suggestions when modal opens
- First suggestion: most recently assigned project, taken from a temporary client last-used cache (in-memory, session-scoped)
- Remaining suggestions: overdue projects, deduplicated against the recent project, capped at 5 total
- When no recent project is cached, show up to 5 overdue projects
- Define fallback behavior when recent assignment or overdue projects unavailable
- Show German empty-state message when no suggestions available

## Capabilities

### New Capabilities
- `assignment-modal-suggestions`: Default suggestions for assignment modal

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New React component logic for suggestion generation + client last-used cache + Rust overdue project query
- Dependencies:
  - BL-022 for project search infrastructure (status filter, timeout, numeric sort)
  - BL-016 for assignment modal where suggestions are displayed
  - BL-032 for the combobox shell the suggestions render into (replaces today's plain `<select>` and provides the free-text search referenced in the empty state)

## Note

Overdue project query (previously scoped to BL-022) is implemented here, as it is only consumed by the default suggestions feature.
Uses a single Daylite call that pairs `{"category": {"equal": "Überfällig"}}` with a status filter for `new_status` and `in_progress`.
The Daylite API has no multi-value operator for scalar fields, so the two statuses are expressed as OR clauses in one request body, same as the BL-022 search.
