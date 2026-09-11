## 0. Preconditions

- [x] 0.1 The "open project in Daylite" change is implemented, so assignment cards carry an action button row and the full-card click has been replaced

## 1. Free the word "highlight"

- [x] 1.1 Rename the `highlight` prop of `TimetableCell` to `isToday`, in the signature, the `Props` interface, the `cellClass` parameter, the `timetable-row.tsx` call site, and the `timetable-cell.spec.tsx` render helper
- [x] 1.2 Confirm `bun test` still passes, so the rename lands as a rename and nothing else

## 2. Highlight key

- [x] 2.1 Write failing tests for the key derived from a `CellEvent`: the Daylite project reference for an assignment, resolved or not, and none for a bare event or an absence
- [x] 2.2 Write a failing test that two events without a project reference do not match, so bare events and absences are not grouped with each other
- [x] 2.3 Add the key derivation and the match test to `src/app/types.ts`

## 3. Marker color

- [x] 3.1 Add a blue marker color token to `src/app.css`, with its own value inside the `corporate` and the `business` theme block
- [x] 3.2 Verify the title stays legible over the marker in both themes, and that the marker is distinguishable from the current day's `bg-primary/10` column tint

## 4. Toggle and marker on the card

- [x] 4.1 Write failing tests asserting the toggle renders on assignment cards including unresolved ones, not on bare or absence cards, and reads as pressed while its card is highlighted
- [x] 4.2 Add the toggle to the card action buttons in `src/app/components/timetable-cell.tsx` with a Lucide icon and a German label, guarded by the event kind rather than by the condition that hides the Daylite deep-link button
- [x] 4.3 Write failing tests for the marker behind the title of matching cards, and for the card's background, category strip and outline surviving it
- [x] 4.4 Render the marker behind the title of cards whose key matches the active one

## 5. Grid state

- [x] 5.1 Write failing tests for the toggle semantics as a pure transition, the way `nextGhostState` is tested: activating on an unhighlighted card replaces the active highlight, activating on a highlighted card clears it
- [x] 5.2 Add the transition as a pure function and hold the active key in `PlanningGridTable`, threading it and the toggle handler through `week-table.tsx` and `timetable-row.tsx` to the cells, along the path `dropPreview` and `draggedUid` take
- [x] 5.3 Write a failing test for the highlight clearing when the grid shows another week
- [x] 5.4 Clear the active key when the rendered week start changes, comparing against the `weekStart` `PlanningGridTable` already derives, the way `TimetableRow` clears its ghost

## 6. Verification

- [x] 6.1 Add a grid-level test with mocked commands covering a highlight across several employees and days of one week, including a matching card in the current day's column
- [x] 6.2 Verify the highlight survives a reload of the visible week and follows a card dragged to another employee or day
- [x] 6.3 Verify a highlight whose matching events are all deleted marks nothing and leaves the grid otherwise unchanged
- [x] 6.4 Verify the week sliding in during a trackpad swipe renders unhighlighted
