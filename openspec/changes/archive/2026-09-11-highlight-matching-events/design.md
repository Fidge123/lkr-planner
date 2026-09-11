## Context

See proposal.md - Why.

Five properties of the current grid shape this design.

Nothing in a `CellEvent` identifies the same work across employees.
`uid` is the UID of one employee's CalDAV event, so it is unique per card and matches nothing else in the grid.
`projectRef` is the Daylite reference (`/v1/projects/42`) and recurs wherever the project is scheduled.
It is set on assignments and null on bare events and absences, which is what makes it both the match key and the test for whether a card can be highlighted at all.

The grid already carries transient state from the root to every card.
`useAppointmentDrag` holds `dropPreview` and `activePayload` in `PlanningGridTable`, and `week-table.tsx` and `timetable-row.tsx` pass them down to `TimetableCell` untouched.
`dropPreview` is rewritten on every pointer move during a drag, so the grid already re-renders wholesale at pointer frequency and a second root-level value costs nothing measurable next to it.

Rows already know how to drop transient state when the week changes.
`TimetableRow` compares the rendered week start against the one its ghost belongs to and clears the ghost when they differ, which works for arrow navigation and for the trackpad swipe alike because both change the week the grid renders.

The card's box is fully spoken for; its title text is not.
A cell's `<td>` uses `ring-2 ring-inset ring-primary` for the drop target and `ring-2 ring-inset ring-error` for an absence conflict.
The card itself carries the neutral surface color, the Daylite category color as a `border-l-4` strip, a brightness filter on hover, and a visibility change while it is lifted mid-drag.
The title inside it carries no treatment at all.

`TimetableCell` already has a prop named `highlight`, which marks the current day's column with `bg-primary/10` and is fed by `highlight={isToday(day)}`.
The word is taken in exactly the file this change edits.

## Goals / Non-Goals

**Goals:**

- One question answered well: where else in this week is the project on this card.
- The highlight outlives the gesture that started it, because the planner acts on what it shows after they stop pointing at the card.
- No new data.
  Everything the match needs already reaches the frontend on `CalendarCellEvent`.
- No persistence.
  The highlight is a reading aid for the session, never written to CalDAV or the local store.

**Non-Goals:**

- Hover, dwell, or drag as triggers.
  All three are transient, and the highlight is wanted while the planner works rather than while they point.
- Bare events.
  They carry no Daylite reference, so the only key available is the title, and two unrelated events both called "Besprechung" would match.
- Absences.
  Their colors already separate the types, and a second visual language over them would compete with the conflict ring.
- Highlighting day headers, the employee column, or the cell background.
  Cards only.
- Matching across weeks.
  The highlight applies to the week on screen.
- Multiple simultaneous highlights.
- A toolbar list of the week's projects to pick from.
  Worth revisiting if picking from a card turns out to be the wrong starting point, but it is a second entry point for a feature that has none yet.

## Decisions

### The Daylite project reference is the key

Two cards match when both carry a `projectRef` and the two references are equal.

The reference is stable across a rename in Daylite, and two projects that happen to share a name stay distinct.
Matching on the title would invert both of those.

A missing reference never matches a missing reference.
Bare events and absences both carry `projectRef: null`, so an equality test alone would group every one of them with all the others.
The exclusion is therefore part of the match rather than a separate guard on the render, and it is the case worth a test more than any other in this change.

An assignment whose project could not be resolved keeps a `projectRef` and shows the raw reference as its title.
It matches on the reference like any other assignment, which is the useful behaviour: every card broken by the same unreachable project lights up together.
This is why the toggle must not reuse the condition that hides the Daylite deep-link button, which is suppressed for exactly these cards.

### An explicit toggle rather than hover

Hover was the starting proposal and is rejected on timing rather than on noise.
The planner hovers a card to learn where its project sits, then moves the pointer to that place to act, and the answer is gone before they arrive.
A toggle survives the pointer leaving, which is the whole point.

The toggle is one more button in the card action button row that the "open project in Daylite" change introduced.
That change replaced the full-card click, which is what makes a per-card action possible at all.
The row exists on assignment cards only, and since only assignments are highlightable, no card needs a row it does not already have.

Exactly one highlight is active.
Activating the toggle on a card whose key is already active clears it, and every highlighted card carries the toggle, so an active highlight always shows at least one visible way to switch it off.
That makes a separate dismiss affordance, an Escape binding or a toolbar control, redundant for now.

### A marker behind the title, not a ring or a background

A matching card gets a colored marker behind its title text.
Its background, its category strip, and its box are untouched, and nothing else in the grid changes.

A ring on the card was the first choice and loses to one thing the card colors do not control.
The category strip is a `border-l-4` painted in a color that comes from Daylite, so any hue reserved for a ring will sit six pixels from an arbitrary hue on some customer's category.
The rendered comparison showed this directly: a yellow ring against an orange category strip smears into one band, and the same collision is available to every other ring color for some other category.
The title text is the only surface on the card that carries no color decision yet, and putting the mark there removes the adjacency entirely.

The marker also composes with the treatments already on the card.
It survives the drop-target and conflict rings on the `<td>`, the hover brightness filter, and the visibility change while a card is lifted, none of which touch the title's own background.
A card background change would not: a grey one inverts between the themes, because `base-300` is lighter than `base-200` in `corporate` and darker in `business`, so the same rule makes matches come forward in one theme and recede in the other.

Dimming the non-matching cards was the other candidate and is deferred, not rejected.
It finds the matches fastest, but it greys the absences along with everything else, it collides with the quick-add ghost at 50% and the lifted card during a drag, and it costs the planner the ability to read the rest of the week while the highlight is up, which is the next thing they need.
A marker composes with dimming, so it can be added later without changing what the highlight means.

### The marker color is defined per theme

The marker is blue, in the same family as `primary`, and is a new token rather than a DaisyUI semantic color.

Blue collides with `primary` for a ring or a background, because `primary` is the drop-target ring and the current day's column wash.
It does not collide for a marker: the mark is on the title text rather than on the card's box, so it never competes with a ring, and it stays legible over the day column's `bg-primary/10` because the marker is far more saturated than that wash.

The token is defined inside each `@plugin "daisyui/theme"` block rather than once in `@theme`, which departs from `--color-absence-vacation` and its siblings.
Those three are single values because they are read as fills behind italic labels of one known contrast.
The marker sits behind the title, which is `base-content`: near-black at 22% lightness in `corporate` and near-white at 85% in `business`.
No single marker color keeps the text legible against both, so the light theme takes a pale blue and the dark theme a deeper one.

### State at the grid root

The active key lives in `PlanningGridTable` and travels to `TimetableCell` on the path `dropPreview` and `draggedUid` already take.
A React context would spare the two intermediate components a prop, at the cost of a second way of moving grid state that the drag path does not use.

The key is cleared by comparing the rendered week start against the one the active key was set in, the way `TimetableRow` already clears its ghost.
`PlanningGridTable` already derives `weekStart` for `useFrozenDuringDrag`, so the comparison needs no new derivation.
This covers arrow navigation and the trackpad swipe without either having to know about the highlight, including the edge-hover navigation that changes the week mid-drag.
The week sliding in during a swipe is a second `WeekTable` that receives no drag props and receives no highlight props either, so it renders unhighlighted without a rule of its own.

Nothing else clears it.
A reload rewrites events but not project references, so the highlight lands on the reloaded cards.
Dragging a highlighted card to another employee or day carries its reference with it, so it stays highlighted where it lands.
A highlight whose key no longer matches anything, because the last matching assignment was deleted, simply marks nothing and is replaced by the next activation.

The existing `highlight` prop on `TimetableCell` is renamed to `isToday` first, in its own step.
It is fed by `isToday(day)` already, `cellClass` takes it as the first of four booleans, and leaving two unrelated meanings of "highlight" in that file would make every later edit in this change harder to read than it needs to be.

## Risks / Trade-offs

- The toggle adds a control to every assignment card in a grid that can hold many.
  The card action button row is sized for this by the change that introduced it, and the mitigation belongs there rather than here.
- A marker on the title alone may be too quiet in a dense week.
  Dimming the non-matching cards is the escalation, and the decision above keeps it available.
- The marker follows the title's line wrapping, so a title that wraps to two lines is marked as two ragged bands rather than one.
  Visible and tolerable; worth revisiting only if the wrap turns out to read as damage rather than as a marker.
- Excluding bare events means a planner who schedules recurring work as bare calendar entries cannot ask this question about it.
  That is a reason to give the work a Daylite project rather than a reason to match on titles.

## Migration Plan

Not applicable.
Nothing is persisted and no stored data changes.

## Open Questions

None.
