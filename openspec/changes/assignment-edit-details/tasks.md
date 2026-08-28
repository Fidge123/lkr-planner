## 1. iCal foundations

- [ ] 1.1 Add a test pinning how the `icalendar` crate returns an escaped `\n` in `DESCRIPTION`, so `classify_event`'s first-line read is known to hold once notes sit below the `daylite:` line; move unescaping into the parse step if it does not
- [ ] 1.2 Add RFC 5545 line folding to `build_ical_payload` and a test that a long property is folded at 75 octets without splitting a multi-byte character
- [ ] 1.3 Emit the properties as `UID`, `DTSTAMP`, `DTSTART`, `DTEND`, `X-LKR-ORDER`, `SUMMARY`, `X-LKR-TITLE`, `DESCRIPTION`, with a test that `can_patch_slot` still accepts a payload whose note is folded

## 2. Title override and note in the backend

- [ ] 2.1 Extend `build_ical_payload` with an optional title override, writing it to `SUMMARY` and the title it replaced to `X-LKR-TITLE`, and writing the project name to `SUMMARY` with no `X-LKR-TITLE` when there is no override
- [ ] 2.2 Extend `build_ical_payload` with the note, written as `daylite:<ref>`, a blank line, then the note, escaped as text
- [ ] 2.3 Parse `X-LKR-TITLE` and the note in `parse_ical_events` and carry them through `RawVEvent`, `classify_event`, and `PendingEvent`, treating the presence of `X-LKR-TITLE` as the marker that `SUMMARY` holds a custom title
- [ ] 2.4 In `resolve_event`, use `SUMMARY` as the card title when the marker is present and the resolved project name when it is not, on the failed-resolution path too, and add the note and the custom-title marker to `CalendarCellEvent`
- [ ] 2.5 Add tests that an assignment with no marker follows a renamed project, that one with a marker keeps its `SUMMARY`, and that dropping the override removes `X-LKR-TITLE` and returns the card to the project's current name
- [ ] 2.6 Add round-trip tests: note with commas, semicolons, backslashes and line breaks; a note written below the reference line by another client; an event with a note still classified as an assignment

## 3. Requested times in the allocator

- [ ] 3.1 Add an optional requested start and end to the day-planning input, and make `plan_slot_updates` write those times for the placed assignment and plan no even split for that day when they are present
- [ ] 3.2 Leave the day's other assignments untouched on such a write, and keep today's behaviour unchanged when no times are requested
- [ ] 3.3 Add allocator tests: requested times are written as given; the rest of the day keeps its times; times outside 08:00-16:00 are accepted; a write without requested times still splits the window

## 4. Adjacent adjustment

- [ ] 4.1 Add an adjustment flag to the same input, emitting `SlotUpdate`s that set the preceding assignment's end to the requested start and the following assignment's start to the requested end
- [ ] 4.2 Find the neighbours in the day's canonical order from `sequence_day`, skipping bare, absence, and holiday events, and skipping a neighbour that `can_patch_slot` rejects
- [ ] 4.3 Refuse the whole write with a German error naming the conflicting assignment when the requested times would move a neighbour's start to or past its end
- [ ] 4.4 Add tests for the three-assignment case, the first and last assignment of a day, a single-assignment day, the refusal, and a day where a bare event sits between two assignments

## 5. Transient by construction

- [ ] 5.1 Add a test that a create on a day previously written with requested times returns every assignment on it to the even split
- [ ] 5.2 Add the same test for a delete, for a reorder, and for a drag of the assignment itself
- [ ] 5.3 Add a test that a save which changes only the project, title, or note re-allocates the day rather than re-sending the times the modal displayed
- [ ] 5.4 Add a test that the written payload carries no property distinguishing an assignment written with requested times from one holding allocated times

## 6. Write paths carry the new fields

- [ ] 6.1 Grow `AssignmentWrite` with `title_override` carrying both the custom title and the title it replaced, and `note`, and fix every construction site the compiler flags
- [ ] 6.2 Extend `CreateAssignmentInput` and `UpdateAssignmentInput` with the new fields, the optional requested times, and the adjustment flag, and thread them through `create_assignment` and `update_assignment`
- [ ] 6.3 Make `move_assignment` carry the source event's title override and note onto the target calendar before deleting the source
- [ ] 6.4 Add a test that a reorder and a re-allocation both leave an event's note and title override intact
- [ ] 6.5 Regenerate the Tauri bindings in `src/generated/tauri.ts`

## 7. Modal: date, times, and the checkbox

- [ ] 7.1 Add the custom-title marker with the replaced title, `note`, and the event's times to `CellEvent` and `toCellEvent`, leaving `title` as the value to display
- [ ] 7.2 Add date state to `use-assignment-modal`, initialised from the cell's day, validated on save with a German error, and passed to the write
- [ ] 7.3 Add start and end time state, initialised from the event's times in edit mode and from the slot the new assignment would receive in create mode, validated so both are filled and end is after start, with German errors
- [ ] 7.4 Send the times as requested times only when the planner touched a time field in this dialog session, and allocate as today otherwise
- [ ] 7.5 Disable the time fields with a German hint for an assignment `can_patch_slot` rejects
- [ ] 7.6 Add the adjacent-adjustment checkbox, ticked by default, sent only alongside requested times
- [ ] 7.7 Render the date and time controls, the checkbox, and the German hint that the times hold until the day is next rearranged, using DaisyUI form controls, Lucide icons, and no nested `div`/`span`

## 8. Modal: title and note

- [ ] 8.1 Add title state, empty when the assignment carries no override, with the resolved project name as the field's placeholder, and left alone when the planner picks a different project
- [ ] 8.2 Send an emptied title field as dropping the override, and any entered title as an override recording the project name it replaces
- [ ] 8.3 Add note state, initialised from the event and passed to the write
- [ ] 8.4 Extend the dirty tracking so date, times, title, and note edits mark the modal changed, while the checkbox alone does not
- [ ] 8.5 Render the title and note fields so the project field's Daylite name stays visible next to the title
- [ ] 8.6 Add modal tests for each scenario in the `assignment-modal-crud` delta

## 9. Category

- [ ] 9.1 Return the project categories from `/categories?entity=project` as a list carrying each category's name, its color, and whether it is still active, sorted by name
- [ ] 9.2 Derive the name-to-color lookup in `services/daylite-categories.ts` from that one list, keeping retired categories in the lookup and out of the picker
- [ ] 9.3 Add a Daylite command that sets a project's category by PATCHing `/projects/<id>` with the picked name, returning the normalized German error when Daylite rejects it
- [ ] 9.4 Replace the cached project for the written reference so the next resolution returns the new category instead of waiting out the cache lifetime, with a test that a project just made fixed is refused by `refuse_protected_event`
- [ ] 9.5 Record a cassette for the category list and the category write, and pin the request shape against it
- [ ] 9.6 Add category state to `use-assignment-modal`, initialised from the assignment's resolved project category and following the project when the planner picks a different one
- [ ] 9.7 Send the category write only when the planner picked a different category, after the calendar write has succeeded, and show the German error in the modal when it fails while keeping the calendar changes
- [ ] 9.8 Extend the dirty tracking so a category change marks the modal changed
- [ ] 9.9 Render the picker with the active categories, each with its name and color swatch, the neutral swatch for a category without a color, and the German hint that the category belongs to the project, using DaisyUI form controls and no nested `div`/`span`
- [ ] 9.10 Add modal tests for each scenario in the `assignment-modal-crud` category delta, including that picking `"Termin FIX geplant"` leaves the reopened modal protected and that picking another category releases it

## 10. Drag paths

- [ ] 10.1 Pass the dragged card's custom title with the title it replaced, and its note, into the reschedule and move writes in `use-appointment-drag`, without requested times or adjacent adjustment
- [ ] 10.2 Add tests that a reschedule and a move both carry the details through and write the standard window

## 11. Verification

- [ ] 11.1 Run `cargo test`, `bun test`, and the Biome check
- [ ] 11.2 Exercise the write path against the disposable Radicale server in `caldav/write.rs` with an assignment carrying a title override, a note, and requested times with adjacent adjustment
- [ ] 11.3 Run `bunx openspec validate assignment-edit-details --strict`
