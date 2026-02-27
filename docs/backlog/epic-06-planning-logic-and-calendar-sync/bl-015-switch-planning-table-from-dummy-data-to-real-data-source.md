# BL-015: Replace Remaining Dummy Assignment Data

## Scope
- Remove remaining dummy assignment usage in weekly planning rows/cells.
- Load assignment state from persistent app data instead of static fixtures.
- Keep existing German loading, empty, and error states consistent.
- Ensure week navigation uses the same persisted source.

## Acceptance Criteria
- Weekly table no longer relies on hardcoded assignment fixtures.
- Reloading the app keeps previously saved assignments visible.
- Empty and error states remain user-friendly and German.

## Dependencies
- Requires stable assignment persistence contract (local store or backend command).

## Out of Scope
- Assignment modal behavior details (covered by BL-016/BL-031/BL-032/BL-033).

## Tests (write first)
- Service tests for load/save assignment persistence.
- UI tests for loading, empty, persisted-data, and error states.
- Week-navigation tests to verify persisted data consistency.
