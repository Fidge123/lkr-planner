## Context

See proposal.md - Why.

Four properties of the current write path shape this design.

An assignment event is rebuilt from scratch on every modal save and every drag.
`build_ical_payload` emits a fixed VEVENT from UID, date, project name, project reference, slot times, and order index; whatever else the resource held is gone after the PUT.
Only slot re-allocation preserves foreign properties, because `patch_event_slot` copies through every line it does not rewrite.

`DESCRIPTION` is load-bearing.
`classify_event` reads the Daylite project reference from the first description line, and an event whose first line does not start with `daylite:` is a bare event, not an assignment.

`SUMMARY` is written but not read back for assignments.
`resolve_event` overwrites the title with the resolved Daylite project name and only falls back to `SUMMARY` when resolution fails, which is what makes a project rename in Daylite show up on the card.

Times are not the event's to keep.
`plan_slot_updates` re-derives every participating assignment's `DTSTART` and `DTEND` from its position in the day and rewrites them on every create, update, and delete on that day.
Whatever a planner types into a time field is gone by the next write on that day unless the assignment stops participating.
The allocator already has a notion of a non-participant: an assignment it cannot rewrite safely is filtered out, keeps its times, and takes no share of the window.

## Goals / Non-Goals

**Goals:**

- One place in the VEVENT for each new field, chosen so a planner editing the event in ZEP or another CalDAV client sees something sensible.
- No write path can silently drop a field: the fields travel together through `AssignmentWrite`, so adding one to the struct forces every caller to supply it.
- Manual times reuse the allocator's existing non-participant path rather than introducing a second notion of who owns an event's times.

**Non-Goals:**

- File attachments.
  Dropped from this change.
- Dragging a card's edges in the grid to resize it.
  Times are edited in the modal only.
- Rearranging a whole day's times at once, or a per-employee working-time profile that replaces the fixed 08:00-16:00 window.
- Overlap detection between an assignment and the employee's bare or absence events.
  The allocator does not do this today and manual times do not make it more pressing.
- Editing bare events or absences.
  They stay read-only, as they are today.

## Decisions

### The custom title lives in `SUMMARY`, `X-LKR-TITLE` keeps the title it replaced

`SUMMARY` carries whatever title the planner wants shown, because that is the property every calendar client reads.
`X-LKR-TITLE` carries the title the custom one replaced, which is the Daylite project name at the moment the planner typed over it.
Its presence is the marker that `SUMMARY` was set by hand: `resolve_event` uses `SUMMARY` when the property is there and the resolved project name when it is not, so an assignment nobody has renamed keeps following the project through a rename in Daylite exactly as today.
Writing an assignment without a custom title emits no `X-LKR-TITLE` and puts the project name in `SUMMARY`, as today.

Holding the replaced title rather than a copy of the custom one is what makes a later reset affordance cheap: the modal can offer "Titel zurücksetzen" and show the planner what it would go back to without resolving anything.
That affordance is not in this change - the planner resets by emptying the field, which drops `X-LKR-TITLE` and restores the project name - but the storage is chosen so adding it later needs no format change.

One consequence to keep in mind: the stored value ages.
A project renamed in Daylite after the planner set a custom title leaves `X-LKR-TITLE` holding the old name.
So it is a record of what was replaced, not a cache of the project name, and nothing reads it as one.
Reset therefore clears the property and falls back to live resolution rather than writing the stored value back into `SUMMARY`, and the stored value is only ever displayed as the fallback when resolution fails.

Alternatives considered.
Mirroring the custom title into both properties makes the marker redundant with `SUMMARY` and throws away the replaced title for no gain.
Storing the custom title in the description below the `daylite:` line mixes it with the planner's note and forces a parsing convention on text a human also edits.
A bare boolean marker property would work for display, but loses the reset affordance for the same number of bytes on the wire.

### The note is the description below the `daylite:` line

`DESCRIPTION` becomes `daylite:<ref>` followed by a blank line and the note.
`classify_event` already reads only the first line, so classification is unaffected, and a note written by another calendar client below the reference line is picked up as the assignment's note.
The note is what remains after dropping the first line and any single blank line that follows it, so a round-trip through the planner does not accumulate blank lines.

Alternatives considered.
An `X-LKR-NOTE` property keeps the description clean but hides the note from every other calendar client, which is the main reason to store it on the event at all.
Moving the project reference to an X-property and giving `DESCRIPTION` entirely to the note would be cleaner, but every assignment already on a calendar carries the reference in the description, so it would need a migration and would break older events.

### Manual times pin an assignment out of the allocation, marked by `X-LKR-TIMES`

An assignment written with times set by hand carries `X-LKR-TIMES:manual`, and `plan_slot_updates` filters it out of `assignments` next to the existing `can_patch_slot` check.
From there the allocator's documented behaviour for a non-participant applies unchanged: it keeps its times, takes no share of the window, and the rest of the day still spreads across the full 08:00-16:00.
That the remaining assignments may overlap a pinned one is not a defect to fix here; it is the same trade-off the module comment already records, and the adjacent-adjustment checkbox is the tool for closing it.

Pinning has to be an explicit marker rather than a comparison against the times the allocator would have produced.
Inferring the pin -- "these times are not the ones the split would give, so they were set by hand" -- flips an assignment back to automatic whenever the day's count changes underneath it and makes the pin depend on how many other assignments exist.

Clearing both time fields removes the property, which is the only way back into the allocation.
Without it the first hand-typed time would pin the assignment forever, and the planner's mental model of a day that arranges itself would have no undo.

Alternatives considered.
A per-employee working-time profile that shifts the window is what the fixed 08:00-16:00 window really wants long term, but it does not give a single job an unusual start, which is what was asked for.
Storing manual times in X-properties beside the allocated `DTSTART`/`DTEND` would keep the allocation intact underneath, but every other calendar client would show the assignment at the allocated time, which defeats the point of setting a time at all.

### Adjacent adjustment is a day-level plan, not a per-event write

The checkbox is handled where the allocation already lives: `plan_slot_updates` gains a mode that, instead of splitting the window, takes the edited assignment's requested times and emits `SlotUpdate`s for the assignment immediately before and immediately after it in the day's order.
The predecessor's end becomes the new start, the successor's start becomes the new end, and both are pinned along with the edited one, because their times no longer follow the split.
Neighbours are found in the day's canonical order from `sequence_day`, so "adjacent" means the adjacent assignment, skipping bare, absence, and holiday events -- consistent with everything else the allocator does.

Both neighbours are written through `patch_event_slot`, which is the existing multi-event write path and preserves foreign properties.
A neighbour the allocator cannot rewrite safely is skipped rather than refused: it was already outside the allocator's reach.

The refusal case is a squeeze, not an overlap: if the new times would move a neighbour's start to or past its own end, the whole save is refused rather than writing a zero-length or inverted event.
Refusing beats clamping because a clamped neighbour silently loses its duration, and beats cascading because a cascade can push a chain of assignments off the end of the day.

Defaulting the checkbox to ticked follows from what the day looks like otherwise: leave it off and the first manual time turns a tidy day into one with a gap and an overlap, which is a surprising result for the planner who only wanted a later start.

### Folding, and the property order it forces

A note is the first free-text field long enough to break the 75-octet line limit, so `build_ical_payload` starts folding instead of writing every property on one physical line.
That interacts with `can_patch_slot`: it refuses an event whose folded continuation follows `DTSTART`, `DTEND`, or `X-LKR-ORDER`, and a refused event drops out of re-allocation silently.
So the payload is emitted as `UID`, `DTSTAMP`, `DTSTART`, `DTEND`, `X-LKR-ORDER`, `X-LKR-TIMES`, `SUMMARY`, `X-LKR-TITLE`, `DESCRIPTION`, with the foldable properties last and `X-LKR-ORDER` ahead of them.

Not folding at all is the alternative, and it is what the code does today.
It is rejected because the length of a note is the planner's to choose, and a server that enforces the limit would reject the write rather than degrade.

### The new fields travel in `AssignmentWrite`

`AssignmentWrite` grows `title_override` (custom title plus the title it replaced), `note: Option<String>`, and `times: AssignmentTimes`, where the times are either allocated or pinned to an explicit start and end.
Every write path already builds one, so the compiler flags each caller that has to be taught to carry the fields through: the modal save, `move_assignment`, and the drag hook's reschedule.

The drag paths have no dialog to read the fields from, so they must carry the values the card already holds.
`CalendarCellEvent` therefore gains the custom-title marker, the note, and the pin, and the drag hook passes them back into the write.
A drag deliberately does not re-run adjacent adjustment: it changes which day an assignment sits on, and adjusting the neighbours of a day the planner is not looking at would be a surprise.

### Reorder needs no change

`reorder_assignment_core` never rebuilds the event; it re-slots the day through `patch_event_slot`, which copies unknown properties through.
A pinned assignment is filtered out of the allocation but keeps its order index, so reordering still moves its card without touching its times.

## Risks / Trade-offs

[A planner pins one assignment and the rest of the day keeps spreading across the full window, so cards overlap] → This is the allocator's documented behaviour for non-participants, the checkbox closes it in the common case, and the day's cards are ordered by order index rather than by time, so an overlap does not reorder the grid.

[A day where every assignment is pinned never re-allocates, so a delete leaves a gap] → Accepted: a planner who set every time by hand is asking to own the day. Nothing is corrupt, and clearing one assignment's times hands it back to the allocator.

[Pinning is inferred rather than marked after an event is edited in ZEP, where a planner changes the times of an assignment that carries no pin] → Not handled: the next write on that day re-allocates it back, as today. Making a foreign edit pin the event would need a comparison this design deliberately rejects.

[The `icalendar` crate may not unescape `\n` in `DESCRIPTION` the way `classify_event`'s `lines()` assumes] → Verified by a test before the note is built on top of it; if it does not, unescaping moves into the parse step.

[Adjacent adjustment writes up to three events, so a partial failure leaves the day half-adjusted] → The neighbours go through `patch_event_slot`, the same multi-event path re-allocation already uses, so the failure mode is the one the planner already has for a failed re-slot: a reload shows the true state.

[The `X-LKR-TIMES` marker is invisible in ZEP, so a planner there cannot tell a pinned assignment from an allocated one] → Not mitigable in the planner; the times themselves are correct in every client, which is what the employee needs.

## Migration Plan

No data migration.
Events written before this change carry no `X-LKR-TITLE`, no note, and no `X-LKR-TIMES`, and every new field is optional, so they load and save exactly as they do today - in particular, every existing assignment stays in the allocation, because a pin only exists once a planner sets one.
An older build of the planner reading an event written by the new one shows the override as the card title only when the project fails to resolve, rewrites the event without the note, and re-allocates a pinned assignment back into the day's split on the next write to that day - so a rollback loses details and manual times on the assignments touched after it.
