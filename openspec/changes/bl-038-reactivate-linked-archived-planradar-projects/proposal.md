## Why

Linked Planradar projects may become archived/closed over time. When assigning employees to these projects, they need to be reactivated first. This ensures assignments work correctly in Planradar.

## What Changes

- Detect archived/closed Planradar projects that are linked
- Implement reactivation/reopen operation via Planradar API
- Log reactivation actions and failures as sync events
- Skip already active projects without side effects

## Capabilities

### New Capabilities
- `planradar-project-reactivation`: Reactivate archived linked projects

### Modified Capabilities
- `ical-sync`: Extended to reactivate projects before sync

## Impact

- Code: New service function for project reactivation
- Dependencies: Depends on BL-009, BL-010, BL-037