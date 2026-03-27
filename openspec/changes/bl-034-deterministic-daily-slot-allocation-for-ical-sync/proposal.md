## Why

iCal sync needs to allocate time slots for employee assignments on a given day. Multiple assignments on one day need deterministic, non-overlapping time slots to export to iCal format. Users expect repeated sync runs to produce identical iCal output for the same assignments.

## What Changes

- Implement pure function to allocate time slots for multiple same-day assignments
- Use fixed daily window `08:00-16:00` (8 hours)
- Split window evenly across assignments sorted by a stable canonical order
- Ensure deterministic output regardless of input order

## Capabilities

### New Capabilities
- `slot-allocation`: Deterministic time slot allocation for same-day assignments

### Modified Capabilities
- `ical-sync`: Modified to use slot allocation for iCal event times

## Impact

- Code: New pure function for slot allocation logic
- Dependencies: Consumed by BL-017 ical-synchronization