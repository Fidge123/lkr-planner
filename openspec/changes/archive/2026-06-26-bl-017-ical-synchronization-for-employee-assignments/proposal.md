## Why

With CalDAV as the source of truth for assignments (BL-015), this change implements the write side: creating, updating, and deleting VEVENT entries in employee primary CalDAV calendars. Writes happen inline with user actions — no background sync queue is needed because the calendar is the store, not a downstream copy.

## What Changes

- Write lkr-planner assignments as VEVENT entries to employee primary CalDAV on create/update/delete
- Ensure idempotent writes via stable VEVENT UID (no duplicate events on retry)
- Never write to absence calendars

## Capabilities

### New Capabilities
- `ical-assignment-sync`: Write assignment events directly to employee primary CalDAV calendars

### Modified Capabilities
- `assignment-persistence`: Extends BL-015 read model with write operations

## Impact

- Code: CalDAV PUT/DELETE commands in Tauri backend; BL-016 modal triggers these commands
- APIs: CalDAV PUT and DELETE to employee primary calendars
- Dependencies: BL-015 for event encoding format; BL-034 for daily slot allocation
