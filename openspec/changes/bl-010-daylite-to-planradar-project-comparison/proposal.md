## Why

The application needs to link Daylite projects with existing Planradar projects. This enables tracking which Planradar project corresponds to each Daylite project, which is essential for synchronization and data consistency across both systems.

## What Changes

- Determine if Daylite project has a linked Planradar project reference
- Allow linking an already existing Planradar project when no link exists
- Persist Planradar project ID into configured Daylite custom field
- Ensure idempotent link behavior across repeated runs

## Capabilities

### New Capabilities
- `planradar-project-linking`: Link existing Planradar projects to Daylite projects

### Modified Capabilities
- `planradar-api-client`: Extends BL-009 with linking functionality

## Impact

- Code: New linking module in Tauri backend
- APIs: Daylite custom field read/write, Planradar project read
- Dependencies: BL-009 Planradar API client