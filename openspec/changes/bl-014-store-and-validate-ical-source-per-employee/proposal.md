## Why

The application uses employee iCal sources for assignment synchronization and absence tracking. Users need validation tooling to ensure iCal URLs are valid and accessible before relying on them for planning data.

## What Changes

- Provide validation tooling for employee primary assignment iCal and secondary absence iCal URLs
- Add a manual connection test action per source URL
- Show clear German feedback for success/failure including actionable hints
- Persist latest validation/test timestamp for transparency in the UI

## Capabilities

### New Capabilities
- `employee-ical-validation`: Validate and test employee iCal sources

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New validation service in Tauri backend
- APIs: iCal URL HTTP requests
- Dependencies: Existing Daylite contact iCal support (BL-008)