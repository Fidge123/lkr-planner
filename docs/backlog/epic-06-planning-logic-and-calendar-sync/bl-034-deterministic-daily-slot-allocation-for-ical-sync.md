# BL-034: Deterministic Daily Slot Allocation for iCal Sync

## Scope
- Define deterministic slot allocation for multiple assignments on one employee/day.
- Use fixed daily window `08:00-16:00` local time.
- Split window evenly across assignments for that day.
- Ensure stable ordering rule so repeated sync runs produce identical times.

## Acceptance Criteria
- Slot allocation is deterministic for same assignment set.
- 1..n same-day assignments always fit into `08:00-16:00` window without overlap.
- Reordered input still produces stable canonical slot output.

## Dependencies
- Consumed by BL-017 sync orchestration.

## Out of Scope
- User-defined custom working time windows.

## Tests (write first)
- Pure function tests for 1..n assignment slot splits.
- Stability tests across repeated runs and reordered input.
- Boundary tests for daylight-saving transitions.
