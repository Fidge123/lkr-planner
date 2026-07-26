## Context

Both dependencies are now shipped and archived, so this change is unblocked.
`drag-drop-appointments` (archived 2026-07-21) delivers coarse drag: a card lands on a day cell and its position within the cell is undefined.
BL-034 (archived 2026-07-19) established `slot-allocation` as a baseline spec.
It splits the fixed 08:00-16:00 window into non-overlapping slots for same-day assignments, ordered deterministically by canonical UID, and guarantees that reordered input produces identical output.

That UID ordering means there is no planner-controllable notion of "which assignment runs first", and therefore no meaningful "drop before/after" target.
This change introduces a persisted order concept so that visual order, drop position, and allocated time slots all agree.

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
BL-034's shipped write format settles the encoding: re-slotting goes through `patch_event_slot`, which rewrites only the VEVENT's DTSTART and DTEND and copies every other line through untouched.
A dedicated X-property (for example `X-LKR-ORDER`) therefore survives every re-slot write without extra handling, so that is the chosen encoding.

Deriving the order from the BL-034-assigned DTSTART sequence is rejected for the same reason as ordering by DTSTART above: slots are derived from the order, so reading the order back out of them is circular.

Two concrete consequences for implementation:
- `RawVEvent` carries no X-properties today, and `parse_ical_events` extracts only uid, summary, description, dtstart, dtend, and times.
  Reading the index needs a new parsed field alongside the `etag` and `raw_ical` fields BL-034 added.
- `can_patch_slot` excludes a resource whose DTSTART or DTEND is folded across physical lines.
  An integer X-property is far short of the 75-octet fold threshold, so writing the index does not push events into that excluded set.

### Re-key BL-034 slot allocation to order-index order
The allocator changes its sort key from UID to the order index, while keeping every other guarantee (fixed 08:00-16:00 window, non-overlapping, deterministic for a given index ordering).
`slot-allocation` is now a baseline spec, so this is authored as a normal MODIFIED delta against it.

The change is confined to two functions in `src-tauri/src/integrations/calendar/slots.rs`:
- `allocate_slots` takes `&[String]` of UIDs and sorts them, so it needs to receive the order index alongside each UID and sort on that instead.
  The UID stays as the tie-breaker so the allocation remains total even if two events momentarily share an index.
- `plan_slot_updates` builds that list and must carry the index through from the parsed event.

Everything else in the allocator is unaffected.
The boundary arithmetic, the "already in its slot" skip that keeps repeated runs from writing, and the `extra_uid` handling for an in-flight create all work off the sorted sequence rather than the sort key itself.

### Before/after placement on drop
The drop handler maps the pointer's position within the target cell to an insertion point between two cards (or at the start/end), then assigns the dragged card an order index at that point and re-sequences the cell.
This reuses the dnd-kit pointer coordinates already available from `drag-drop-appointments`; it extends the drop dispatch rather than replacing it.

## Risks / Trade-offs

- [Visual order and slot times drift if only one is re-keyed] → Both read the same persisted index; the allocator re-key task and the sort-by-index rendering task land together.
- [Assignments excluded from re-slotting still occupy a render position] → `slot-allocation` leaves an assignment untouched when its times cannot be rewritten safely, meaning a DURATION-based end, more than one VEVENT in the resource, a folded DTSTART or DTEND, or a missing resource URL.
  Such an assignment still has an order index and still renders in the cell, but keeps whatever times it already had, so its card can show times that disagree with its position.
  The order index stays the single source of truth for position, and only the times are stale for that one card.
- [Re-sequencing churn writes many events on each reorder] → Per-day card counts are small, so revisit fractional indices only if write volume becomes a problem.
- [Re-sequencing multiplies CalDAV writes per gesture] → A reorder rewrites the index on every card in the day and the allocator then re-slots the same day, so one drag can touch every event in a cell.
  The allocator's "already in its slot" skip limits the second pass to events whose times actually changed.

## Open Questions

- Whether to adopt fractional indices to avoid neighbor rewrites on insert.
- Insertion-point hit-testing granularity within a cell (midpoint split per card vs. gap zones).
- Whether an assignment excluded from re-slotting should be visually marked, given its times can disagree with its position.
