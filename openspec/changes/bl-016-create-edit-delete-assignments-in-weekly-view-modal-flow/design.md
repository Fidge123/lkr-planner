## Context

This change implements the assignment modal for creating, editing, and deleting assignments. It builds on the persistent assignment storage from BL-015 and provides the UI interaction layer.

## Goals / Non-Goals

**Goals:**
- Reliable modal open/close from cell clicks
- Support create, edit, delete operations
- Immediate UI update after save
- Keyboard and cancel handling

**Non-Goals:**
- Suggestion ranking (covered by BL-031)
- Search result filtering (covered by BL-032)
- Next-day quick-add (covered by BL-033)

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
**Decision**: Optimistic UI update with backend confirmation
- Update UI immediately on save
- Roll back on error with German error message

## Risks / Trade-offs

- **Risk**: Unsaved changes lost on accidental close
  - **Mitigation**: Confirm dialog for unsaved changes

- **Risk**: Concurrent edits from multiple windows
  - **Mitigation**: Last-write-wins with timestamp