## Why

After assigning a project to a day, showing a translucent copy in the next day helps users quickly continue their planning without opening the modal again.

## What Changes

- After saving an assignment, show translucent copy in next day cell for same employee
- Allow one-click add from translucent item to create real assignment
- Clearly distinguish translucent items from persisted assignments
- Remove/update translucent suggestions when source assignment changes

## Capabilities

### New Capabilities
- `next-day-quick-add`: Translucent quick-add suggestion for next day

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New React component for translucent suggestion rendering and interaction
- Dependencies: Depends on BL-016 assignment modal CRUD baseline
