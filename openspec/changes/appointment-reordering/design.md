## Context

`drag-drop-appointments` delivers coarse drag: a card lands on a day cell, keeps its time-of-day, and its position within the cell is undefined.
BL-034 (`slot-allocation`) splits the fixed 08:00-16:00 window into non-overlapping slots for same-day assignments, ordered deterministically by canonical UID, and explicitly guarantees that reordered input produces identical output.
That UID ordering means there is currently no planner-controllable notion of "which assignment runs first" — and therefore no meaningful "drop before/after" target.

This change introduces a persisted order concept so that visual order, drop position, and allocated time slots all agree.
It depends on both `drag-drop-appointments` (the drag machinery and `move_assignment`) and BL-034 (the slot allocator it re-keys).

## Goals / Non-Goals

**Goals:**
- Give each same-day, same-employee assignment a stable, planner-controlled order index.
- Let a drag reorder a card within its day without changing date or employee.
- Let cross-day and cross-employee drops land precisely before or after a target card.
- Render each cell sorted by order index.
- Re-key BL-034 slot allocation to assign slots in order-index order.

**Non-Goals:**
- Free-form time editing (slots remain BL-034's fixed-window split).
- Reordering across days in a single gesture beyond what drag already supports.
- Changing the coarse drag behavior shipped in `drag-drop-appointments`.

## Decisions

### Order index as the single source of truth for within-day order
A per-assignment integer (or fractional) order index defines position among same-day, same-employee siblings.
Visual sort, drop placement, and BL-034 slot assignment all read this index, so the three never diverge.
On any write that changes a day's membership (create, delete, reorder, cross-day/cross-employee move), the affected day(s) are re-sequenced to a dense 0..n-1 ordering and persisted.

Alternatives considered:
- Order by DTSTART (the slot times themselves): circular, since slots are derived from order; rejected.
- Fractional indices to avoid re-sequencing neighbors on every insert: viable optimization, but dense re-sequencing is simpler and the per-day card count is small; deferred unless needed.

### Storage of the order index
The index is carried per VEVENT so it survives across devices via CalDAV, consistent with how project references already live in the event.
Exact encoding (a dedicated X-property on the VEVENT vs. deriving order from the BL-034-assigned DTSTART sequence) is settled during implementation against BL-034's final write format; the requirement is only that order is persisted and shared, not how.

### Re-key BL-034 slot allocation to order-index order
BL-034's allocator changes its sort key from UID to the order index, while keeping every other guarantee (fixed 08:00-16:00 window, non-overlapping, deterministic for a given index ordering).
This is a `slot-allocation` delta, but `slot-allocation` is not yet a baseline spec — it lives only inside the unarchived BL-034 change.
A MODIFIED delta cannot be authored against a non-existent baseline, so this change is sequenced after BL-034 is archived; until then the re-key is tracked as a task and a coordination note here, and the proposal lists the modified capability for intent.

### Before/after placement on drop
The drop handler maps the pointer's position within the target cell to an insertion point between two cards (or at the start/end), then assigns the dragged card an order index at that point and re-sequences the cell.
This reuses the dnd-kit pointer coordinates already available from `drag-drop-appointments`; it extends the drop dispatch rather than replacing it.

## Risks / Trade-offs

- [Sequencing dependency on BL-034] → This change is explicitly blocked until BL-034 is implemented and archived; the proposal and tasks call this out, and nothing here ships before then.
- [Visual order and slot times drift if only one is re-keyed] → Both read the same persisted index; the BL-034 re-key task and the sort-by-index rendering task land together.
- [Re-sequencing churn writes many events on each reorder] → Per-day card counts are small; revisit fractional indices only if write volume becomes a problem.
- [Two changes touching slot allocation] → `drag-drop-appointments` deliberately does not touch ordering, so the only allocator change comes from here, after BL-034.

## Open Questions

- Final order-index encoding on the VEVENT vs. deriving it from BL-034's persisted DTSTART order.
- Whether to adopt fractional indices to avoid neighbor rewrites on insert.
- Insertion-point hit-testing granularity within a cell (midpoint split per card vs. gap zones).
