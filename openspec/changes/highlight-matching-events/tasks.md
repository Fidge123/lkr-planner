## 0. Preconditions

- [ ] 0.1 The "open project in Daylite" change is implemented, so cards carry an action button row and the full-card click has been replaced

## 1. Highlight key

- [ ] 1.1 Write failing tests for the key derived from a `CellEvent`: the project reference for an assignment, the trimmed title for a bare event, none for an absence, and no match between an assignment and a bare event that read alike
- [ ] 1.2 Add the key derivation to `src/app/types.ts`

## 2. Highlight color

- [ ] 2.1 Add a theme-level color token for the highlight ring to `src/app.css`, in the light and the dark theme
- [ ] 2.2 Verify its separation from the drop-target blue, the conflict red, and the three absence hues in both themes with the dataviz palette checks

## 3. Toggle on the card

- [ ] 3.1 Write failing tests asserting the toggle renders on assignment and bare cards, not on absence cards, and reads as pressed while its card is highlighted
- [ ] 3.2 Add the toggle to the card action buttons in `src/app/components/timetable-cell.tsx` with a Lucide icon and a German label
- [ ] 3.3 Write failing tests for the ring on matching cards and for the card's own background and category strip surviving it
- [ ] 3.4 Render the ring on cards whose key matches the active one

## 4. Grid state

- [ ] 4.1 Write failing tests for the toggle semantics: activating on an unhighlighted card replaces the active highlight, activating on a highlighted card clears it
- [ ] 4.2 Hold the active key in `PlanningGridTable` and thread it and the toggle handler through `week-table.tsx` and `timetable-row.tsx` to the cells, along the path `dropPreview` and `draggedUid` take
- [ ] 4.3 Write a failing test for the highlight clearing when the grid shows another week
- [ ] 4.4 Clear the active key when the rendered week start changes, the way `TimetableRow` clears its ghost

## 5. Verification

- [ ] 5.1 Add a grid-level test with mocked commands covering a highlight across several employees and days of one week
- [ ] 5.2 Verify the highlight survives a reload of the visible week and follows a card dragged to another employee or day
- [ ] 5.3 Verify a highlight whose matching events are all deleted marks nothing and leaves the grid otherwise unchanged
