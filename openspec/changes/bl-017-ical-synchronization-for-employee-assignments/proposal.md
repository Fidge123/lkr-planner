## Why

Employee assignments need to be synchronized to their personal iCal calendars so they can see their work schedule in their preferred calendar application. This enables employees to view their assignments outside the planning application.

## What Changes

- Synchronize assignment create/update/delete operations to employee primary iCal
- Ensure idempotent sync behavior (no duplicate events across repeated runs)
- Track and expose sync status per assignment for troubleshooting
- Keep absence iCal strictly read-only input

## Capabilities

### New Capabilities
- `ical-assignment-sync`: Synchronize assignments to employee iCal calendars

### Modified Capabilities
- `assignment-persistence`: Extends with sync status tracking

## Impact

- Code: New sync orchestration service in Tauri backend
- APIs: iCal push/update operations to employee calendars
- Dependencies: BL-034 for deterministic daily slot allocation