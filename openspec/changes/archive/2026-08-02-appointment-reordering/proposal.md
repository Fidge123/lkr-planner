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
- `slot-allocation`: Allocate same-day time slots in order-index order rather than by canonical UID, so slot position follows the planner-controlled order.

## Impact

- Both dependencies are satisfied: `drag-drop-appointments` (archived 2026-07-21) supplies the drag machinery and `move_assignment`, and BL-034 (archived 2026-07-19) makes `slot-allocation` a baseline spec with a working allocator.
- Frontend: drop logic gains before/after position detection within a cell; `TimetableCell` renders sorted by order index; new intra-day reorder handling on the dnd-kit drop dispatch.
- Backend: assignment writes persist and re-sequence the order index for the affected day(s); `allocate_slots` consumes the index as its sort key instead of the UID.
- Data: order index stored as an X-property on each lkr-planner VEVENT, which survives re-slotting because `patch_event_slot` rewrites only DTSTART and DTEND.
