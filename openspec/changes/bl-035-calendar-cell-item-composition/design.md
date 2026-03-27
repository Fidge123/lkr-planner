## Context

The calendar cell must display a mix of items from different sources with different characteristics. We need a unified model to simplify rendering logic and ensure consistent behavior.

## Goals / Non-Goals

**Goals:**
- Normalize all item types into common model
- Preserve source-specific data in extended fields
- Flag read-only items appropriately
- Sort project items by start time

**Non-Goals:**
- Visual rendering and interaction details
- Editing logic for any item type

## Decisions

### Normalized Item Model
**Decision**: Create `CalendarCellItem` interface with common fields:
- `id`: Unique identifier
- `type`: 'absence' | 'holiday' | 'assignment' | 'appointment'
- `title`: Display title
- `startTime`: Optional time for sorting
- `isReadOnly`: Boolean flag for editable state
- `sourceData`: Type-specific original data

### Source Composition
**Decision**: Composition function combines all sources:
1. Fetch all-day absences from absence calendar
2. Fetch holidays (with German names from BL-027)
3. Fetch project assignments for employee/day
4. Fetch preexisting appointments

### Ordering Rule
**Decision**: Sort by item type then start time
- Order: absences first, then holidays, then appointments, then assignments
- Within same type: sort by start time (ascending)
- Assignments without start time at end of their group

### Read-Only Flag
**Decision**: Set `isReadOnly = true` for:
- All absences (from external calendar)
- All holidays (derived from Nager API)
- All preexisting appointments

Assignments are editable (isReadOnly = false).

## Risks / Trade-offs

- **Risk**: Multiple items overflow cell height
  - **Mitigation**: Pagination/scrolling handled in rendering layer, not composition

- **Risk**: Different date formats across sources
  - **Norm**: Convert all dates to ISO string in composition layer

- **Risk**: Performance with many sources
  - **Mitigation**: Composition runs on cell render; cache composed results