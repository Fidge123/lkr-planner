## Context

This change implements iCal synchronization for employee assignments. When assignments are created, updated, or deleted, corresponding events are pushed to the employee's primary iCal calendar. The system must be idempotent to prevent duplicate events.

## Goals / Non-Goals

**Goals:**
- Sync assignment changes to employee primary iCal
- Ensure no duplicate events on repeated sync
- Track sync status per assignment
- Never write to absence calendars

**Non-Goals:**
- Manual global sync trigger UX (out of scope)

## Decisions

### Idempotency Strategy
**Decision**: Use assignment UUID as iCal UID
- Each assignment gets a stable UUID
- iCal events use same UUID across syncs
- Repeated syncs update existing events instead of creating duplicates

### Sync Timing
**Decision**: Sync immediately on assignment change
- Trigger sync when assignment is saved/deleted
- Queue sync operations for reliability
- Background processing to not block UI

### Status Tracking
**Decision**: Store sync status with assignment
- Track: pending, synced, failed
- Store last sync timestamp
- Expose status for troubleshooting UI

## Risks / Trade-offs

- **Risk**: iCal server unavailable
  - **Mitigation**: Queue failed syncs for retry

- **Risk**: Large number of assignments to sync
  - **Mitigation**: Batch operations, show progress