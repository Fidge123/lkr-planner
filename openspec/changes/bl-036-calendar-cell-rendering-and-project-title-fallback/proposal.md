## Why

The calendar cell needs to render all composed items with appropriate behavior. Project titles may come from multiple sources (custom, Planradar, Daylite) with a specific fallback order. Preexisting appointments need time and title display.

## What Changes

- Render all calendar cell items from BL-035 composition
- Implement project title fallback chain: custom → Planradar → Daylite company (single) → Daylite project
- Enforce read-only behavior for non-editable items
- Display start/end times for all items

## Capabilities

### New Capabilities
- `calendar-cell-rendering`: Render composed calendar cell items
- `project-title-fallback`: Determine project display name from multiple sources

### Modified Capabilities
- `calendar-cell-composition`: Extended to include title fallback

## Impact

- Code: New React components for cell rendering, fallback logic function
- Dependencies: Depends on BL-035 calendar-cell-composition