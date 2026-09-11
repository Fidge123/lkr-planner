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

- Assignment cards gain a highlight toggle among the card action buttons.
  Activating it marks every other card in the visible week that carries the same project.
- Matching is by Daylite project reference, so a project renamed in Daylite still matches and two projects sharing a name never do.
- Bare events and absence cards get no toggle and are never highlighted.
  A bare event carries no Daylite reference, so its title is the only thing it could be matched by, and titles group unrelated events that happen to read alike.
  Absence types already have their own colors, so an absence is found by looking rather than by asking.
- A highlighted card is marked by a colored marker behind its title, the way a text marker is drawn over a word on paper.
  The card's box is left alone.
- One highlight is active at a time.
  Activating the toggle on a card of another project replaces it; activating it again on a card that is already highlighted clears it.
- The highlight spans every employee row and every visible day of the current week, and marks cards only.
  Day headers and the employee column are unaffected.
- Navigating to another week clears the highlight, by arrow or by trackpad swipe.
  Reloading assignments does not, because the highlight is keyed by project reference rather than by event UID, so it survives every event the reload rewrites.

## Capabilities

### New Capabilities

- `event-highlighting`: marking every card in the visible week that shares the Daylite project of a card the planner picked.

### Modified Capabilities

None.

## Impact

- Depends on the card action buttons introduced by the "open project in Daylite" change, which replaced the full card click.
  That change has landed, and assignment cards already carry the row.
  This change adds one button to it and does not introduce the row itself.
  Bare and absence cards have no such row and do not gain one, because they are not highlightable.
- `src/app/types.ts`: a highlight key derived from a `CellEvent`, which is the Daylite project reference for an assignment and null for everything else, plus a match test that never treats two absent keys as equal.
- `src/app/page.tsx`: the active highlight key as root state, threaded to the cells along the path `dropPreview` and `draggedUid` already take, and cleared when the week changes.
- `src/app/components/week-table.tsx` and `src/app/components/timetable-row.tsx`: pass the active key and the toggle handler through.
- `src/app/components/timetable-cell.tsx`: the toggle button on assignment cards, and the marker behind the title of cards whose key matches.
  The existing `highlight` prop, which marks the current day's column, is renamed to `isToday` so the word names one thing in this file.
- `src/app.css`: one color token for the title marker, defined per theme rather than once, because a single value cannot stay legible behind both the light theme's dark title text and the dark theme's light title text.
- No backend change.
  `projectRef` already reaches the frontend on every `CalendarCellEvent`, and the highlight is never persisted.
- Tests: `bun test` coverage for the key derivation, the null-key exclusion, the toggle semantics, the week-change clearing, and the rendered marker.
