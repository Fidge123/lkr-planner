## Context

This change implements the assignment modal for creating, editing, and deleting assignments. It builds on the persistent assignment storage from BL-015 and provides the UI interaction layer.

## Goals / Non-Goals

**Goals:**
- Reliable modal open/close from cell clicks
- Support create, edit, delete operations
- Grid reload after save
- Keyboard and cancel handling
- Simple project picker using BL-022 queries

**Non-Goals:**
- Suggestion ranking (covered by BL-031)
- Live text filter with debounce and keyboard navigation (covered by BL-032)
- Next-day quick-add (covered by BL-033)
- Same-day slot redistribution (covered by BL-034 / BL-017)

## Decisions

### Modal Trigger
**Decision**: Click on employee/day cell opens modal
- Clear user affordance for interaction
- Pre-select cell context in modal

### Edit Flow
**Decision**: Same modal handles create and edit
- Detect if assignment exists for cell
- Pre-populate form for editing
- Single code path for both operations

### State Update
**Decision**: Reload after save (not optimistic)
- Call `reloadAssignments()` after successful save or delete
- No local state patching; always reflects confirmed backend state
- On error, modal stays open and shows German error message

### CalDAV Resource URL
**Decision**: Parse and store `d:href` from CalDAV REPORT responses
- `parse_caldav_report` currently discards the resource URL per event
- Change to return `(href, RawVEvent)` pairs and surface href in `CalendarCellEvent`
- Required for `PUT` (update) and `DELETE` operations which use the resource URL, not only the UID

### Event Time Window
**Decision**: New and updated assignments use the fixed `08:00–16:00` window
- A single assignment for a day occupies the full window
- Same as BL-034's single-assignment case; no redistribution logic needed here
- BL-034's slot-splitting algorithm applies during BL-017 iCal sync only

### CalDAV Write URL Convention
**Decision**: Assume ZEP follows standard CalDAV conventions
- Create: `PUT {calendar_base_url}/{uid}.ics` with a new UUID
- Update: `PUT {href}` using the stored resource URL
- Delete: `DELETE {href}` using the stored resource URL
- Quirks discovered during manual testing are fixed as they arise

### Project Picker
**Decision**: Simple filtered list via BL-022 query service
- Uses `daylite-project-query` capability from BL-022
- Filters to `new_status` and `in_progress` projects only
- No debounce, no keyboard navigation — those are added by BL-032

## Risks / Trade-offs

- **Risk**: Unsaved changes lost on accidental close
  - **Mitigation**: Confirm dialog for unsaved changes

- **Risk**: Concurrent edits from multiple windows
  - **Mitigation**: Last-write-wins with timestamp