## Why

The calendar cell needs to display multiple types of items: absences, holidays, project assignments, and preexisting appointments. Each source has different properties and edit capabilities. We need a unified item model that composes all sources for consistent rendering.

## What Changes

- Define normalized `CalendarCellItem` model with common fields
- Implement composition function that aggregates all sources
- Add read-only flag for non-editable items (absences, holidays, appointments)
- Sort project entries by start time within the cell

## Capabilities

### New Capabilities
- `calendar-cell-composition`: Compose calendar cell items from multiple sources

### Modified Capabilities
- `week-view`: Modified to use composed cell items from new model

## Impact

- Code: New composition function and type definitions
- Dependencies: Depends on BL-027 holiday import