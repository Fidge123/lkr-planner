## 1. iCal foundations

- [ ] 1.1 Add a test pinning how the `icalendar` crate returns an escaped `\n` in `DESCRIPTION`, so `classify_event`'s first-line read is known to hold once notes sit below the `daylite:` line; move unescaping into the parse step if it does not
- [ ] 1.2 Add RFC 5545 line folding to `build_ical_payload` and a test that a long property is folded at 75 octets without splitting a multi-byte character
- [ ] 1.3 Emit the properties as `UID`, `DTSTAMP`, `DTSTART`, `DTEND`, `X-LKR-ORDER`, `X-LKR-TIMES`, `SUMMARY`, `X-LKR-TITLE`, `DESCRIPTION`, with a test that `can_patch_slot` still accepts a payload whose note is folded

## 2. Title override and note in the backend

- [ ] 2.1 Extend `build_ical_payload` with an optional title override, writing it to `SUMMARY` and the title it replaced to `X-LKR-TITLE`, and writing the project name to `SUMMARY` with no `X-LKR-TITLE` when there is no override
- [ ] 2.2 Extend `build_ical_payload` with the note, written as `daylite:<ref>`, a blank line, then the note, escaped as text
- [ ] 2.3 Parse `X-LKR-TITLE` and the note in `parse_ical_events` and carry them through `RawVEvent`, `classify_event`, and `PendingEvent`, treating the presence of `X-LKR-TITLE` as the marker that `SUMMARY` holds a custom title
- [ ] 2.4 In `resolve_event`, use `SUMMARY` as the card title when the marker is present and the resolved project name when it is not, on the failed-resolution path too, and add the note and the custom-title marker to `CalendarCellEvent`
- [ ] 2.5 Add tests that an assignment with no marker follows a renamed project, that one with a marker keeps its `SUMMARY`, and that dropping the override removes `X-LKR-TITLE` and returns the card to the project's current name
- [ ] 2.6 Add round-trip tests: note with commas, semicolons, backslashes and line breaks; a note written below the reference line by another client; an event with a note still classified as an assignment

## 3. Manual times in the allocator

- [ ] 3.1 Write `X-LKR-TIMES:manual` in `build_ical_payload` for a pinned assignment, with the planner's start and end as `DTSTART`/`DTEND`, and omit the property otherwise
- [ ] 3.2 Parse the marker into `RawVEvent` and `PendingEvent`, and surface the pin and the times on `CalendarCellEvent`
- [ ] 3.3 Filter pinned assignments out of the participating set in `plan_slot_updates`, next to the existing `can_patch_slot` check, and confirm they keep their order index through `sequence_day`
- [ ] 3.4 Add allocator tests: a pinned assignment keeps its times across a re-allocation; the unpinned ones still split the full window; a day of only pinned assignments produces no updates; releasing a pin returns the assignment to the split
- [ ] 3.5 Add a test that deleting a pinned assignment re-allocates the remaining ones

## 4. Adjacent adjustment

- [ ] 4.1 Add a mode to `plan_slot_updates` that, given an edited assignment's requested times, emits `SlotUpdate`s pinning the preceding assignment's end to the new start and the following assignment's start to the new end
- [ ] 4.2 Find the neighbours in the day's canonical order from `sequence_day`, skipping bare, absence, and holiday events, and skipping a neighbour that `can_patch_slot` rejects
- [ ] 4.3 Refuse the whole write with a German error naming the conflicting assignment when the requested times would move a neighbour's start to or past its end
- [ ] 4.4 Add tests for the three-assignment case, the first and last assignment of a day, a single-assignment day, the refusal, and a day where a bare event sits between two assignments

## 5. Write paths carry the new fields

- [ ] 5.1 Grow `AssignmentWrite` with `title_override` carrying both the custom title and the title it replaced, `note`, and times that are either allocated or pinned, and fix every construction site the compiler flags
- [ ] 5.2 Extend `CreateAssignmentInput` and `UpdateAssignmentInput` with the new fields plus the adjacent-adjustment flag, and thread them through `create_assignment` and `update_assignment`
- [ ] 5.3 Make `move_assignment` carry the source event's title override, note, and pin onto the target calendar before deleting the source
- [ ] 5.4 Add a test that a reorder leaves an event's pin, note, and title override intact and does not replace pinned times with the slot the new position would have
- [ ] 5.5 Add a test that a same-day re-allocation triggered by a neighbouring write leaves a pinned assignment's times and details intact
- [ ] 5.6 Regenerate the Tauri bindings in `src/generated/tauri.ts`

## 6. Modal: date, times, and the checkbox

- [ ] 6.1 Add the custom-title marker with the replaced title, `note`, the pin, and the event's times to `CellEvent` and `toCellEvent`, leaving `title` as the value to display
- [ ] 6.2 Add date state to `use-assignment-modal`, initialised from the cell's day, validated on save with a German error, and passed to the write
- [ ] 6.3 Add start and end time state, initialised from the event's current times, validated so end is after start and both are filled or both empty, with German errors
- [ ] 6.4 Treat both fields empty as releasing the pin, and any entered pair as pinning the assignment
- [ ] 6.5 Disable the time fields with a German hint for an assignment `can_patch_slot` rejects
- [ ] 6.6 Add the adjacent-adjustment checkbox, ticked by default, passed to the write only when a time actually changed
- [ ] 6.7 Render the date, time, and checkbox controls in `assignment-modal.tsx` with German labels, DaisyUI form controls, Lucide icons, and no nested `div`/`span`

## 7. Modal: title and note

- [ ] 7.1 Add title state, initialised from the card's title, following a newly picked project only while it has not been edited by hand, recording the project name it replaces when the planner types over it, and dropping the override when emptied
- [ ] 7.2 Add note state, initialised from the event and passed to the write
- [ ] 7.3 Extend the dirty tracking so date, times, title, and note edits mark the modal changed, while the checkbox alone does not
- [ ] 7.4 Render the title and note fields alongside the existing project picker
- [ ] 7.5 Add modal tests for each scenario in the `assignment-modal-crud` delta

## 8. Drag paths

- [ ] 8.1 Pass the dragged card's custom title with the title it replaced, its note, and its pin into the reschedule and move writes in `use-appointment-drag`, without running adjacent adjustment
- [ ] 8.2 Add tests that a reschedule and a move both carry the details through, and that a pinned card keeps its times at the target

## 9. Verification

- [ ] 9.1 Run `cargo test`, `bun test`, and the Biome check
- [ ] 9.2 Exercise the write path against the disposable Radicale server in `caldav/write.rs` with an assignment carrying a title override, a note, and pinned times
- [ ] 9.3 Run `bunx openspec validate assignment-edit-details --strict`
