## 0. Preconditions

- [ ] 0.1 Confirm `drag-drop-appointments` is implemented (coarse drag and `move_assignment`)
- [ ] 0.2 Confirm BL-034 is implemented and archived so `slot-allocation` is a baseline spec that can be modified

## 1. Order index model

- [ ] 1.1 Write failing tests for persisting and re-sequencing the per-day order index on create, delete, reorder, and move
- [ ] 1.2 Implement persistence of the order index per assignment and dense re-sequencing of affected day(s)
- [ ] 1.3 Sort each cell's cards by order index in the grid render

## 2. Re-key BL-034 slot allocation

- [ ] 2.1 Write failing tests asserting slots are assigned in order-index order and that changing the index changes the slot
- [ ] 2.2 Change the BL-034 allocator's sort key from UID to the order index, keeping the fixed window and non-overlap guarantees
- [ ] 2.3 Verify visual order and allocated times agree across create/delete/reorder/move

## 3. Intra-day reorder via drag

- [ ] 3.1 Write failing tests for reordering a card within its cell without changing date or employee
- [ ] 3.2 Extend the dnd-kit drop dispatch to handle same-cell reorder by setting the order index and re-sequencing

## 4. Precise before/after placement on cross-cell drops

- [ ] 4.1 Write failing tests for drop-before, drop-after, and drop-into-empty-area placement in the target cell
- [ ] 4.2 Implement insertion-point hit-testing within a cell and set the dragged card's order index accordingly
- [ ] 4.3 Re-sequence and persist the target cell after placement

## 5. Verification

- [ ] 5.1 Add a grid-level test covering reorder-within-day and precise cross-employee placement (mocked commands)
- [ ] 5.2 Manually verify intra-day reorder, before/after landing, and that allocated times follow the visual order
- [ ] 5.3 Run `bun lint`, `bun format`, `bun test`, and `cargo test`; fix issues until all green
