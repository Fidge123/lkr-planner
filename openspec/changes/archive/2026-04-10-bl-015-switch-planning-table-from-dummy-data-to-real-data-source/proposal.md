## Why

The weekly planning table currently uses dummy/hardcoded assignment data. This prevents persistent planning - users lose their assignments when the app restarts. This change replaces dummy data with real persistent storage.

## What Changes

- Remove remaining dummy assignment usage in weekly planning rows/cells
- Load assignment state from persistent app data instead of static fixtures
- Keep existing German loading, empty, and error states consistent
- Ensure week navigation uses the same persisted source

## Capabilities

### New Capabilities
- `assignment-persistence`: Store and load assignment data persistently

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New CalDAV REPORT reading in Tauri backend; frontend hook replacing dummy data
- Storage: Employee primary CalDAV calendars (ZEP) — no additions to LocalStore
- Dependencies: BL-017 for CalDAV write operations (create/edit/delete)