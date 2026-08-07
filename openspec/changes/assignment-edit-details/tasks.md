## 1. iCal foundations

- [ ] 1.1 Add a test pinning how the `icalendar` crate returns an escaped `\n` in `DESCRIPTION`, so `classify_event`'s first-line read is known to hold once notes sit below the `daylite:` line; move unescaping into the parse step if it does not
- [ ] 1.2 Add RFC 5545 line folding to `build_ical_payload` and a test that a long property is folded at 75 octets without splitting a multi-byte character
- [ ] 1.3 Reorder the emitted properties to `UID`, `DTSTAMP`, `DTSTART`, `DTEND`, `X-LKR-ORDER`, `SUMMARY`, `X-LKR-TITLE`, `DESCRIPTION`, `ATTACH`, with a test that `can_patch_slot` still accepts a payload carrying a folded property

## 2. Title override and note in the backend

- [ ] 2.1 Extend `build_ical_payload` with an optional title override, writing it to `SUMMARY` and the title it replaced to `X-LKR-TITLE`, and writing the project name to `SUMMARY` with no `X-LKR-TITLE` when there is no override
- [ ] 2.2 Extend `build_ical_payload` with the note, written as `daylite:<ref>`, a blank line, then the note, escaped as text
- [ ] 2.3 Parse `X-LKR-TITLE` and the note in `parse_ical_events` and carry them through `RawVEvent`, `classify_event`, and `PendingEvent`, treating the presence of `X-LKR-TITLE` as the marker that `SUMMARY` holds a custom title
- [ ] 2.4 In `resolve_event`, use `SUMMARY` as the card title when the marker is present and the resolved project name when it is not, on the failed-resolution path too, and add the note and the custom-title marker to `CalendarCellEvent`
- [ ] 2.5 Add tests that an assignment with no marker follows a renamed project, that one with a marker keeps its `SUMMARY`, and that dropping the override removes `X-LKR-TITLE` and returns the card to the project's current name
- [ ] 2.6 Add round-trip tests: note with commas, semicolons, backslashes and line breaks; a note written below the reference line by another client; an event with a note still classified as an assignment

## 3. Attachments in the backend

- [ ] 3.1 Add an attachment type carrying file name, content type, and bytes, and write each one as an `ATTACH;ENCODING=BASE64;VALUE=BINARY;FMTTYPE=;FILENAME=` property in `build_ical_payload`
- [ ] 3.2 Parse `ATTACH` properties back into that type, falling back to a name derived from the content type when `FILENAME` is missing
- [ ] 3.3 Enforce the 5 MB total cap before the PUT and return a German error naming the cap when it is exceeded
- [ ] 3.4 Add a command that returns one attachment's bytes for a given event href and attachment index, and a command that writes them to a temp file and opens it through the opener plugin, both with German error messages
- [ ] 3.5 Add round-trip tests: file name, content type, and bytes unchanged; several attachments on one event; an event already over the cap still parses

## 4. Write paths carry the new fields

- [ ] 4.1 Grow `AssignmentWrite` with `title_override` carrying both the custom title and the title it replaced, plus `note` and `attachments`, and fix every construction site the compiler flags
- [ ] 4.2 Extend `CreateAssignmentInput` and `UpdateAssignmentInput` with the new fields and thread them through `create_assignment` and `update_assignment`
- [ ] 4.3 Make `move_assignment` re-read the source event and carry its title override, note, and attachments onto the target calendar before deleting the source
- [ ] 4.4 Add a test that a reorder through `patch_event_slot` leaves an event's attachment, note, and title override intact
- [ ] 4.5 Add a test that a same-day slot re-allocation triggered by a neighbouring write leaves an assignment carrying an attachment intact
- [ ] 4.6 Regenerate the Tauri bindings in `src/generated/tauri.ts`

## 5. Modal: date, title, and note

- [ ] 5.1 Add the custom-title marker with the replaced title, `note`, and the attachment metadata list to `CellEvent` and `toCellEvent`, leaving `title` as the value to display
- [ ] 5.2 Add date state to `use-assignment-modal`, initialised from the cell's day, validated on save with a German error, and passed to the write
- [ ] 5.3 Add title state, initialised from the card's title, following a newly picked project only while it has not been edited by hand, recording the project name it replaces when the planner types over it, and dropping the override when emptied
- [ ] 5.4 Add note state, initialised from the event and passed to the write
- [ ] 5.5 Extend the dirty tracking so date, title, note, and attachment edits all trigger the unsaved-changes dialog
- [ ] 5.6 Render the date, title, and note fields in `assignment-modal.tsx` with German labels, DaisyUI form controls, and no nested `div`/`span`
- [ ] 5.7 Add modal tests for each scenario in the `assignment-modal-crud` delta

## 6. Modal: attachments

- [ ] 6.1 Add attachment state to `use-assignment-modal`: the list read from the event, files added in this session, and removals, all applied only on save
- [ ] 6.2 Check a picked file against the remaining budget and show a German message naming the cap and the rejected file's size
- [ ] 6.3 Render the attachment list with name, size, an open action, and a remove action, using Lucide icons
- [ ] 6.4 Wire the open action to the backend command and surface its failure as a German error without closing the modal
- [ ] 6.5 Add modal tests for each scenario in the `assignment-attachments` spec

## 7. Drag paths

- [ ] 7.1 Pass the dragged card's custom title with the title it replaced, and its note, into the reschedule write in `use-appointment-drag`, and its attachment metadata into the move
- [ ] 7.2 Add tests that a reschedule and a move both carry the details through

## 8. Verification

- [ ] 8.1 Run `cargo test`, `bun test`, and the Biome check
- [ ] 8.2 Exercise the write path against the disposable Radicale server in `caldav/write.rs` with an assignment carrying a title override, a note, and an attachment
- [ ] 8.3 Run `bunx openspec validate assignment-edit-details --strict`
