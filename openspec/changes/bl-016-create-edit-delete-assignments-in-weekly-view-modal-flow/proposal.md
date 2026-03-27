## Why

Users need a modal interface to create, edit, and delete assignments in the weekly planning view. The current workflow lacks a proper UI for assignment management - users can only see assignments but cannot modify them through the application.

## What Changes

- Open assignment modal when user clicks an employee/day cell
- Support assigning and removing projects for the selected employee/day
- Support editing an existing assignment directly from the same modal
- Keep flow optimized for day-based planning (no free time input in baseline)
- Persisted state updates immediately in weekly grid after save

## Capabilities

### New Capabilities
- `assignment-modal-crud`: Modal-based assignment create, edit, delete functionality

### Modified Capabilities
- `assignment-persistence`: Extends BL-015 with modal integration

## Impact

- Code: New React modal component + Tauri commands
- Dependencies: BL-015 for persistent assignment state