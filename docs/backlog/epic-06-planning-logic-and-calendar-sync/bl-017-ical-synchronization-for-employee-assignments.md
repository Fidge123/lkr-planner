# BL-017: iCal Synchronization for Employee Assignments

## Scope
- Mirror assignment create/update/delete changes to employee primary assignment iCal.
- Idempotent synchronization (no duplicate appointments).
- Weekly view remains day-based (no exact time input).
- iCal events use a fixed daily dummy window `08:00-16:00` (local time).
- If an employee has multiple projects on the same day, split this window evenly across those projects.
- Secondary absence iCal is read-only input source; assignment events are never written to absence calendars.

## Acceptance Criteria
- New/Update/Delete in planning creates correct iCal action.
- Sync status per assignment is viewable.
- Slot splitting is deterministic and stable for repeated syncs.

## Tests (write first)
- Sync service tests including retry scenarios.
- Tests for same-day slot splitting (1..n assignments/day).
- Tests ensuring no write operations target absence calendars.
