## Why

Planning a future week means answering "where else is this project already scheduled" over and over.
The grid gives no way to ask it.
A planner reads the week employee by employee and reconstructs a project's footprint from the card titles, which is exactly the kind of scanning the grid was supposed to remove.

The Daylite category color on a card's left strip does not answer it either.
Two different projects in the same Daylite category share a strip color, so the strip narrows the search without ever finishing it.

Hovering a card is the obvious trigger and the wrong one.
A hover highlight fires while the pointer sweeps across cards that were never the question, and it disappears at the moment it becomes useful: the planner moves the pointer away from the card to act on what the highlight just told them.
An explicit toggle on the card is invoked deliberately, stays up while the planner works, and is dismissed deliberately.

## What Changes

- Assignment and bare event cards gain a highlight toggle among the card action buttons.
  Activating it marks every other card in the visible week that carries the same work.
- Matching is by Daylite project reference for assignment cards, so a project renamed in Daylite still matches and two projects sharing a name never do.
- Matching is by event title for bare events, which carry no Daylite reference and have nothing else to be identified by.
- An assignment and a bare event never match each other, even when the project name and the event title are identical.
- Absence cards get no toggle and are never highlighted.
  Absence types already have their own colors, so an absence is found by looking rather than by asking.
- One highlight is active at a time.
  Activating the toggle on a card of another project or title replaces it; activating it again on a card that is already highlighted clears it.
- The highlight spans every employee row and every visible day of the current week, and marks cards only.
  Day headers and the employee column are unaffected.
- Navigating to another week clears the highlight, by arrow or by trackpad swipe.
  Reloading assignments does not, because the highlight is keyed by project reference or title rather than by event UID, so it survives every event the reload rewrites.

## Capabilities

### New Capabilities

- `event-highlighting`: marking every card in the visible week that shares the project or title of a card the planner picked.

### Modified Capabilities

None.

## Impact

- Depends on the card action buttons introduced by the "open project in Daylite" change, which replace the full card click.
  This change adds one button to that row and does not introduce the row itself.
- `src/app/types.ts`: a highlight key derived from a `CellEvent`, kind-scoped so assignments and bare events cannot collide, and null for absences.
- `src/app/page.tsx`: the active highlight key as root state, threaded to the cells along the path `dropPreview` and `draggedUid` already take, and cleared when the week changes.
- `src/app/components/week-table.tsx` and `src/app/components/timetable-row.tsx`: pass the active key and the toggle handler through.
- `src/app/components/timetable-cell.tsx`: the toggle button on assignment and bare cards, and the ring on cards whose key matches.
- `src/app.css`: one theme-level color token for the highlight, reserved for this purpose and separated from the drop-target blue, the conflict red, and the three absence hues in both themes.
- No backend change.
  `projectRef` and `title` already reach the frontend on every `CalendarCellEvent`, and the highlight is never persisted.
- Tests: `bun test` coverage for the key derivation, the kind scoping, the toggle semantics, the week-change clearing, and the rendered ring.
