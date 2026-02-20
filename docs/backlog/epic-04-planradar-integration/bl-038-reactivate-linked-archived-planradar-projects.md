# BL-038: Reactivate Linked Archived Planradar Projects

## Scope
- Detect linked Planradar projects that are archived/closed.
- Reactivate/reopen these linked projects before assignment usage.
- Log reactivation actions and failures as sync events.

## Acceptance Criteria
- Archived linked projects are reopened when reactivation action runs.
- Active linked projects are skipped without side effects.
- Failures are visible for targeted rerun.

## Dependencies
- Depends on BL-009 and link availability from BL-010/BL-037.

## Out of Scope
- Creating new Planradar projects for unlinked cases.

## Tests (write first)
- Service tests for archived/active/not-found scenarios.
- Tests for idempotent no-op behavior on already active projects.
- Logging tests for success and failure paths.
