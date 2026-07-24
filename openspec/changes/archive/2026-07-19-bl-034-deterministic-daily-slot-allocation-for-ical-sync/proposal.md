## Why

BL-017 writes each assignment to CalDAV inline as an independent VEVENT with a fixed 08:00-16:00 window.
When an employee has more than one assignment on the same day, the events fully overlap (all 08:00-16:00), which looks wrong in any calendar client and in the planning grid time display.

The original BL-034 design assumed a batch iCal export step that could see all of a day's assignments at once.
That step does not exist: writes are inline and per-event.
This change re-scopes slot allocation to fit the inline-write model: each assignment write re-distributes the day's assignments into non-overlapping slots and persists the updated times.

## What Changes

- Add a deterministic pure function that splits the fixed 08:00-16:00 window into non-overlapping slots for N same-day assignments, ordered by a stable canonical key (UID)
- On every assignment create, update, and delete (BL-017), re-allocate slots across all of the affected employee's lkr-planner assignments on the affected day(s) and persist the new DTSTART/DTEND
- When an update moves an assignment to a different day, re-allocate both the source and target day
- Keep bare/absence/holiday events untouched (only lkr-planner assignments are re-slotted)

## Capabilities

### New Capabilities
- `slot-allocation`: Deterministic time slot allocation for same-day assignments

### Modified Capabilities
- `ical-assignment-sync`: Assignment writes re-allocate same-day slots instead of using a fixed full-day window

## Impact

- Code: New pure slot-allocation function in the Rust backend; create/update/delete write paths re-slot the affected day(s)
- Dependencies: BL-017 for the CalDAV write infrastructure that this re-slotting drives
