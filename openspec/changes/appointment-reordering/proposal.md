## Why

The `drag-drop-appointments` change moves assignment cards between days and employees but always lands a card on the day cell without controlling its position relative to existing cards.
Planners also want to control the order of an employee's assignments within a day (which one runs first) and to drop a card precisely before or after a specific existing card, since order drives the allocated time slots.

## What Changes

- Introduce a persisted per-assignment order index that defines the position of an assignment among its same-day, same-employee siblings.
- Allow dragging an assignment to reorder it within a day (its cell), changing only its order index.
- Extend cross-day and cross-employee drops (from `drag-drop-appointments`) to land precisely before or after a target card, setting the order index accordingly.
- Render each cell's cards sorted by order index so the visual order matches the persisted order.
- Make BL-034 slot allocation assign time slots in order-index order instead of UID order, so the earliest card in a cell gets the earliest slot.

## Capabilities

### New Capabilities
- `appointment-reordering`: Persisted order index for same-day assignments, intra-day reorder via drag, and precise before/after placement on cross-day and cross-employee drops.

### Modified Capabilities
- `slot-allocation`: Allocate same-day time slots in order-index order rather than by canonical UID, so slot position follows the planner-controlled order. This delta can only be authored once BL-034 is archived and `slot-allocation` is a baseline spec; see design.md.

## Impact

- Depends on `drag-drop-appointments` (coarse cross-day/cross-employee drag) and BL-034 (the `slot-allocation` capability must exist and be implemented).
- Frontend: drop logic gains before/after position detection within a cell; `TimetableCell` renders sorted by order index; new intra-day reorder handling on the dnd-kit drop dispatch.
- Backend: assignment writes persist and re-sequence the order index for the affected day(s); BL-034 slot allocation consumes the index as its sort key.
- Data: order index stored alongside each lkr-planner assignment (CalDAV VEVENT property or derived from persisted DTSTART order); exact storage decided in design.
