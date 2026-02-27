# BL-035: Calendar Cell Item Composition

## Scope
- Compose cell item list from these sources:
  - all-day absence calendar items (read-only)
  - vacation/holiday entries (read-only, German holiday name)
  - project assignments
  - preexisting appointments (read-only)
- Define normalized item model used by cell renderer.
- Define item ordering rule with projects sorted by start time.

## Acceptance Criteria
- All required item types appear in composed cell model.
- Read-only items are flagged as non-editable in model.
- Project entries are sorted by start time.
- Preexisting appointments include start time and title in model.

## Dependencies
- Depends on BL-027 holiday import.

## Out of Scope
- Visual rendering details and interactions.

## Tests (write first)
- Unit tests for source-to-model mapping for each item type.
- Unit tests for sort order and read-only flags.
- Integration tests for mixed-source day composition.
