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
Whatever a planner types into a time field is therefore gone by the next write on that day, and that is the accepted behaviour here rather than a problem to solve.

## Goals / Non-Goals

**Goals:**

- One place in the VEVENT for each new field, chosen so a planner editing the event in ZEP or another CalDAV client sees something sensible.
- No write path can silently drop a field: the fields travel together through `AssignmentWrite`, so adding one to the struct forces every caller to supply it.
- The allocator keeps sole ownership of an assignment's times; entering times by hand steers one write rather than creating a second, competing source of truth.

**Non-Goals:**

- File attachments.
  Dropped from this change.
- Times that outlive the day's next rearrangement.
  A planner's times apply to the write being made and are re-derived away by the next re-allocation, by decision rather than by oversight.
- Dragging a card's edges in the grid to resize it.
  Times are edited in the modal only.
- Rearranging a whole day's times at once, or a per-employee working-time profile that replaces the fixed 08:00-16:00 window.
- Overlap detection between an assignment and the employee's bare or absence events.
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

### Times entered by hand steer one write and are stored nowhere

A write carries an optional set of requested times.
When it does, `plan_slot_updates` writes those times for the edited assignment and plans no even split for that day; when it does not, the day is allocated exactly as it is today.
Nothing is written to the event to say the times came from a planner, so the next create, delete, reorder, or drag on that day re-derives every assignment's times from the split and the entered times are gone.

This is the scope the change is built for, and it is worth being explicit about what it buys.
The whole `X-LKR-TIMES` marker disappears, and with it the parsing, the preservation across write paths, the "release the pin" affordance, the drag that has to decide whether to keep a pin, and the days that stop rearranging themselves because every assignment on them opted out.
A day always arranges itself the same way, which is the property the allocator was built to guarantee.

The trade-off is that entered times are quietly lost later, which the modal has to say out loud rather than let a planner discover.
Hence the German hint next to the time fields; without it the feature reads as a broken pin rather than a deliberate one-shot.

The discriminator is whether the planner touched a time field in this dialog session, not whether the entered times differ from the allocated ones.
Comparing values would make a save that happens to match the split behave differently from one that does not, for no reason the planner could see.

Alternatives considered.
Pinning the assignment out of the allocation with a marker property is what an earlier revision of this design did; it keeps entered times indefinitely at the cost of every consequence listed above, and days where every assignment is pinned stop re-allocating entirely.
A per-employee working-time profile that shifts the window is the better long-term answer to "this employee starts at 07:00", but it does not give a single job an unusual time, which is what was asked for.

### Adjacent adjustment is a day-level plan, not a per-event write

The checkbox is handled where the allocation already lives: `plan_slot_updates` gains a mode that, instead of splitting the window, takes the edited assignment's requested times and emits `SlotUpdate`s for the assignment immediately before and immediately after it in the day's order.
The predecessor's end becomes the new start and the successor's start becomes the new end.
Neighbours are found in the day's canonical order from `sequence_day`, so "adjacent" means the adjacent assignment, skipping bare, absence, and holiday events -- consistent with everything else the allocator does.

Both neighbours are written through `patch_event_slot`, which is the existing multi-event write path and preserves foreign properties.
A neighbour the allocator cannot rewrite safely is skipped rather than refused: it was already outside the allocator's reach.

The refusal case is a squeeze, not an overlap: if the new times would move a neighbour's start to or past its own end, the whole save is refused rather than writing a zero-length or inverted event.
Refusing beats clamping because a clamped neighbour silently loses its duration, and beats cascading because a cascade can push a chain of assignments off the end of the day.

Defaulting the checkbox to ticked follows from what the day looks like otherwise: leave it off and an entered time turns a tidy day into one with a gap and an overlap, which is a surprising result for the planner who only wanted a later start.
The fitted neighbours are as transient as the times that caused them, so the day's next rearrangement tidies everything back to the split at once.

### Folding, and the property order it forces

A note is the first free-text field long enough to break the 75-octet line limit, so `build_ical_payload` starts folding instead of writing every property on one physical line.
That interacts with `can_patch_slot`: it refuses an event whose folded continuation follows `DTSTART`, `DTEND`, or `X-LKR-ORDER`, and a refused event drops out of re-allocation silently.
So the payload is emitted as `UID`, `DTSTAMP`, `DTSTART`, `DTEND`, `X-LKR-ORDER`, `SUMMARY`, `X-LKR-TITLE`, `DESCRIPTION`, with the foldable properties last and `X-LKR-ORDER` ahead of them.

Not folding at all is the alternative, and it is what the code does today.
It is rejected because the length of a note is the planner's to choose, and a server that enforces the limit would reject the write rather than degrade.

### The new fields travel in `AssignmentWrite`

`AssignmentWrite` grows `title_override` (custom title plus the title it replaced) and `note: Option<String>`.
Every write path already builds one, so the compiler flags each caller that has to be taught to carry the fields through: the modal save, `move_assignment`, and the drag hook's reschedule.

Requested times do not belong there.
They are not a property of the assignment but an instruction to one write, so they ride on the command input alongside the adjacent-adjustment flag and reach `plan_slot_updates`, never the payload builder's field list.

The drag paths have no dialog to read the preserved fields from, so they must carry the values the card already holds.
`CalendarCellEvent` therefore gains the custom-title marker and the note, and the drag hook passes them back into the write.
A drag never carries requested times, which is why `appointment-drag-drop` needs no spec change: a drop writes the standard window, exactly as its spec says today.

### Reorder needs no change

`reorder_assignment_core` never rebuilds the event; it re-slots the day through `patch_event_slot`, which copies unknown properties through, so a title override and a note survive it.
Times entered by hand do not, and are not meant to.

## Risks / Trade-offs

[A planner sets a time, a colleague adds an assignment to that day, and the time is silently gone] → The behaviour is deliberate, so the mitigation is telling the truth in the UI: the German hint next to the time fields says the times hold until the day is next rearranged. If planners still find it surprising in use, the pinned design is the escalation, not a patch on this one.

[Entering a time leaves the day with a gap and an overlap] → The checkbox closes it and is ticked by default, and the day's cards are ordered by order index rather than by time, so an overlap does not reorder the grid.

[A planner reads the entered times as a promise to the employee, who then sees the assignment move back to its allocated slot in ZEP] → Same mitigation and the same escalation path; nothing in the calendar records the intent, so the modal is the only place this can be communicated.

[The `icalendar` crate may not unescape `\n` in `DESCRIPTION` the way `classify_event`'s `lines()` assumes] → Verified by a test before the note is built on top of it; if it does not, unescaping moves into the parse step.

[Adjacent adjustment writes up to three events, so a partial failure leaves the day half-adjusted] → The neighbours go through `patch_event_slot`, the same multi-event path re-allocation already uses, so the failure mode is the one the planner already has for a failed re-slot: a reload shows the true state.

[Skipping the even split for one write leaves a day overlapping until something re-allocates it] → That is the feature working; the state is legal iCal and the next ordinary write on that day tidies it.

## Migration Plan

No data migration.
Events written before this change carry no `X-LKR-TITLE` and no note, both are optional, and nothing about how times are stored changes, so existing assignments load, save, and allocate exactly as they do today.
An older build of the planner reading an event written by the new one shows the override as the card title only when the project fails to resolve and rewrites the event without the note, so a rollback loses titles and notes on the assignments touched after it.
Entered times need no rollback story at all: they were never stored as anything but ordinary `DTSTART` and `DTEND`.
