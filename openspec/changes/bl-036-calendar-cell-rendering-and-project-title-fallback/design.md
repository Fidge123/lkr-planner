## Context

The calendar cell must display items with correct rendering and title resolution. Title fallback is critical for user experience when project names vary across systems.

## Goals / Non-Goals

**Goals:**
- Render all item types from composed model
- Show read-only items as non-interactive
- Show editable items with click-to-edit
- Apply title fallback order correctly
- Display start/end times for all items

**Non-Goals:**
- Editing logic (handled by modal in BL-016)
- Cell layout and scrolling (rendering container)

## Decisions

### Title Fallback Order
**Decision**: Apply fallback in exact order:
1. Custom name (if set)
2. Planradar project name
3. Daylite company (only if exactly one linked company)
4. Daylite project name (fallback when no single company)

### Rendering Per Item Type
**Decision**: Each item type renders differently:
- **Absence**: Read-only, shows "Abwesenheit" + type, no edit interaction
- **Holiday**: Read-only, shows German holiday name, no edit interaction
- **Assignment**: Clickable, opens edit modal, shows project title + times
- **Appointment**: Read-only, shows title + start/end time, no edit interaction

### Time Display
**Decision**: Show times for all items with times
- Assignments: show start-end time
- Appointments: show start-end time
- Absences/Holidays: all-day indicator (no specific time)

## Risks / Trade-offs

- **Risk**: User confusion if fallback shows unexpected name
  - **Mitigation**: Document fallback order; add tooltip showing source

- **Risk**: Daylite company count changes during render
  - **Mitigation**: Cache company list, refresh on project link change

- **Risk**: Many items overflow cell
  - **Mitigation**: Cell handles overflow with scroll; pagination if needed
