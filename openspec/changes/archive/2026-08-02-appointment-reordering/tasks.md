## 0. Preconditions

- [x] 0.1 `drag-drop-appointments` is implemented and archived (coarse drag and `move_assignment`)
- [x] 0.2 BL-034 is implemented and archived, so `slot-allocation` is a baseline spec that can be modified

## 1. Order index model

- [x] 1.1 Write failing tests for persisting and re-sequencing the per-day order index on create, delete, reorder, and move
- [x] 1.2 Parse the order index from the VEVENT into `RawVEvent` and carry it through `classify_event` to `PendingEvent` and `CalendarCellEvent`
- [x] 1.3 Write the order index as an X-property in `build_ical_payload`, and preserve it on re-slot writes (`patch_event_slot` already copies unknown lines through)
- [x] 1.4 Re-sequence the affected day(s) to a dense 0..n-1 ordering on every membership change and persist it
- [x] 1.5 Sort each cell's cards by order index in the grid render

## 2. Re-key slot allocation

- [x] 2.1 Write failing tests asserting slots are assigned in order-index order and that changing the index changes the slot
- [x] 2.2 Change `allocate_slots` in `src-tauri/src/integrations/calendar/slots.rs` to sort by order index with the UID as tie-breaker, and carry the index through `plan_slot_updates`
- [x] 2.3 Confirm the fixed window, non-overlap, "already in its slot" skip, and `extra_uid` behaviour are unchanged by the re-key
- [x] 2.4 Verify visual order and allocated times agree across create/delete/reorder/move

## 3. Intra-day reorder via drag

- [x] 3.1 Write failing tests for reordering a card within its cell without changing date or employee
- [x] 3.2 Extend the dnd-kit drop dispatch to handle same-cell reorder by setting the order index and re-sequencing

## 4. Precise before/after placement on cross-cell drops

- [x] 4.1 Write failing tests for drop-before, drop-after, and drop-into-empty-area placement in the target cell
- [x] 4.2 Implement insertion-point hit-testing within a cell and set the dragged card's order index accordingly
- [x] 4.3 Re-sequence and persist the target cell after placement

## 4b. Drop preview

- [x] 4b.1 Write failing tests for the insertion index derived from the cards' geometry, including that it is stable once the preview is spliced in
- [x] 4b.2 Make the day cell the only drop zone so it keeps its outline while the pointer is over one of its cards
- [x] 4b.3 Render a placeholder at the resolved position and lift the dragged card out of the cell's layout so the preview costs no height

## 5. Verification

- [x] 5.1 Add a grid-level test covering reorder-within-day and precise cross-employee placement (mocked commands)
- [x] 5.2 Cover an assignment that is excluded from re-slotting: it keeps its order position but its times are not rewritten
- [x] 5.3 Manually verify intra-day reorder, before/after landing, and that allocated times follow the visual order
- [x] 5.4 Run `bun lint`, `bun format`, `bun test`, and `cargo test`; fix issues until all green
