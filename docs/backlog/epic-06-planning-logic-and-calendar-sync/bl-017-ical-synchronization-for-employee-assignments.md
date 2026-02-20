# BL-017: iCal Assignment Sync Orchestration

## Scope
- Synchronize assignment create/update/delete operations to employee primary iCal.
- Ensure idempotent sync behavior (no duplicate events across repeated runs).
- Track and expose sync status per assignment for troubleshooting.
- Keep absence iCal strictly read-only input.

## Acceptance Criteria
- Create/update/delete actions generate corresponding iCal operations.
- Re-running sync does not duplicate events.
- Each assignment exposes current sync status.
- No write operation targets absence calendars.

## Dependencies
- Depends on BL-034 for deterministic daily slot allocation.

## Out of Scope
- Manual global sync trigger UX.

## Tests (write first)
- Sync service tests for create/update/delete and retry behavior.
- Idempotency tests across repeated sync executions.
- Tests ensuring absence calendars are never written.
