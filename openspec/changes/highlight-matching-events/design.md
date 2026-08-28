## Context

See proposal.md - Why.

Four properties of the current grid shape this design.

Nothing in a `CellEvent` identifies the same work across employees.
`uid` is the UID of one employee's CalDAV event, so it is unique per card and matches nothing else in the grid.
`projectRef` is the Daylite reference (`/v1/projects/42`) and recurs wherever the project is scheduled.
`title` is the resolved Daylite project name for an assignment and the iCal `SUMMARY` for a bare event.

The grid already carries transient state from the root to every card.
`useAppointmentDrag` holds `dropPreview` and `activePayload` in `PlanningGridTable`, and `week-table.tsx` and `timetable-row.tsx` pass them down to `TimetableCell` untouched.
`dropPreview` is rewritten on every pointer move during a drag, so the grid already re-renders wholesale at pointer frequency and a second root-level value costs nothing measurable next to it.

Rows already know how to drop transient state when the week changes.
`TimetableRow` compares the rendered week start against the one its ghost belongs to and clears the ghost when they differ, which works for arrow navigation and for the trackpad swipe alike because both change the week the grid renders.

A cell's `<td>` already uses `ring-2 ring-inset ring-primary` for the drop target and `ring-2 ring-inset ring-error` for an absence conflict.
Both sit on the cell, not the card, so a ring on a card is unclaimed.
A ring is a box shadow and costs no reflow, which matters in WKWebView, where a reflow on re-slotted cards is what `week-table.tsx` already works around with a forced repaint.

## Goals / Non-Goals

**Goals:**

- One question answered well: where else in this week is the work on this card.
- The highlight outlives the gesture that started it, because the planner acts on what it shows after they stop pointing at the card.
- No new data.
  Everything the match needs already reaches the frontend on `CalendarCellEvent`.
- No persistence.
  The highlight is a reading aid for the session, never written to CalDAV or the local store.

**Non-Goals:**

- Hover, dwell, or drag as triggers.
  All three are transient, and the highlight is wanted while the planner works rather than while they point.
- Highlighting day headers, the employee column, or the cell background.
  Cards only.
- Matching across weeks.
  The highlight applies to the week on screen.
- Absences.
  Their colors already separate the types, and a second visual language over them would compete with the conflict ring.
- Multiple simultaneous highlights.
- A toolbar list of the week's projects to pick from.
  Worth revisiting if picking from a card turns out to be the wrong starting point, but it is a second entry point for a feature that has none yet.

## Decisions

### Kind-scoped highlight key

The match key is derived from the event kind and one field, and two events match when their keys are equal:

| Kind | Key | Rationale |
| --- | --- | --- |
| `assignment` | the Daylite project reference | Stable across a rename in Daylite, and two projects that happen to share a name stay distinct. |
| `bare` | the trimmed title | A bare event carries no Daylite reference; its `SUMMARY` is all it has. |
| `absence` | none, never highlightable | Excluded by decision, see Non-Goals. |

The kind is part of the key, so a bare event titled "Bauprojekt Nord" does not match assignments for the project of that name.
The two are different things that happen to read alike, and merging them would make the highlight answer a question nobody asked.

An assignment whose project could not be resolved keeps a `projectRef` and shows the raw reference as its title.
It matches on the reference like any other assignment, which is the useful behaviour: every card broken by the same unreachable project lights up together.

Titles are compared after trimming and otherwise exactly.
Bare events that repeat carry the same `SUMMARY` verbatim, so normalising case or whitespace beyond a trim would only start matching events a planner did not mean to group.

### An explicit toggle rather than hover

Hover was the starting proposal and is rejected on timing rather than on noise.
The planner hovers a card to learn where its project sits, then moves the pointer to that place to act, and the answer is gone before they arrive.
A toggle survives the pointer leaving, which is the whole point.

The toggle is one more button in the card action button row that the "open project in Daylite" change introduces.
That change replaces the full-card click, which is what makes a per-card action possible at all: the card is a `<button>` today, so a second control inside it would be a nested button.

Exactly one highlight is active.
Activating the toggle on a card whose key is already active clears it, and every highlighted card carries the toggle, so an active highlight always shows at least one visible way to switch it off.
That makes a separate dismiss affordance, an Escape binding or a toolbar control, redundant for now.

### Emphasis by ring, not by dimming the rest

Matching cards get a ring in a reserved color.
Everything else in the grid is left alone.

Dimming the non-matching cards would make the matches pop harder in a dense week, and it was the stronger option on legibility alone.
It is deferred because it collides with three treatments that already use opacity for something else: the quick-add ghost at 50%, the lifted card during a drag, and the absence colors, which are themselves opacity-graded to separate the vacation and sick families.
A ring composes with all of them, and dimming can be added later without changing what the highlight means.

The ring color is a new theme-level token rather than one of the DaisyUI semantic colors.
`primary` is the drop-target ring, `error` is the conflict ring, and `warning` already marks an unconfirmed calendar connection in the employee column.
`accent` is unused in the grid but sits at hue 36 in the dark theme, close enough to the dark theme's error red at hue 30 to read as a warning.
A dedicated token follows the precedent of `--color-absence-vacation` and its siblings, which were added for the same reason, and lets the hue be chosen for separation from the drop-target blue, the conflict red, and the three absence hues in both themes.

### State at the grid root

The active key lives in `PlanningGridTable` and travels to `TimetableCell` on the path `dropPreview` and `draggedUid` already take.
A React context would spare the two intermediate components a prop, at the cost of a second way of moving grid state that the drag path does not use.

The key is cleared by comparing the rendered week start against the one the active key was set in, the way `TimetableRow` already clears its ghost.
This covers arrow navigation and the trackpad swipe without either having to know about the highlight, including the edge-hover navigation that changes the week mid-drag.

Nothing else clears it.
A reload rewrites events but not project references or titles, so the highlight lands on the reloaded cards.
Dragging a highlighted card to another employee or day carries its key with it, so it stays highlighted where it lands.
A highlight whose key no longer matches anything, because the last matching assignment was deleted, simply marks nothing and is replaced by the next activation.

## Risks / Trade-offs

- The toggle adds a control to every assignment and bare card in a grid that can hold many.
  The card action button row is sized for this by the change that introduces it, and the mitigation belongs there rather than here.
- A ring alone may be too quiet in a dense week.
  Dimming the non-matching cards is the escalation, and the decision above keeps it available.
- Title matching for bare events is as good as the titles are.
  Two unrelated events both called "Besprechung" will match, which is a visible and self-correcting surprise rather than a silent one.

## Migration Plan

Not applicable.
Nothing is persisted and no stored data changes.

## Open Questions

None.
