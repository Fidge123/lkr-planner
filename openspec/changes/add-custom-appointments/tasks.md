# Tasks

## 1. Confirm the marker survives ZEP

- [ ] 1.1 Write a VEVENT carrying `X-LKR-KIND:custom` to a real ZEP calendar, edit it there, read it back via REPORT and verify the property is still present; if it is not, stop and revisit the marker decision in design.md before continuing
- [ ] 1.2 Write ADR 0015 in `docs/adr` recording the X-property marker, the classification order and the rejected DESCRIPTION-prefix alternative, and verify it follows the format of the existing ADRs

## 2. Backend classification

- [ ] 2.1 Write failing `cargo test`s in `ical.rs` and `events/classify.rs`: the marker is parsed into `RawVEvent`, an event with only the marker classifies as custom, a `daylite:` reference wins over the marker, and an event with neither stays bare
- [ ] 2.2 Replace `PendingEvent.project_ref` with an `EventLink` enum (`Project`, `Custom`, `Bare`), add `CalendarEventKind::Custom`, map custom events in `resolve_event` with no project status, category or reference, and verify the tests from 2.1 and the existing `events` tests pass
- [ ] 2.3 Write a failing test that `project_references` skips custom events so no Daylite lookup is made for them, then satisfy it

## 3. Backend writes

- [ ] 3.1 Write failing `ical.rs` tests: a custom payload carries the title as SUMMARY, the marker, no `daylite:` DESCRIPTION, and round-trips a title with commas, semicolons, backslashes and umlauts unchanged
- [ ] 3.2 Introduce `AssignmentContent` (`project` / `custom`) and make `build_ical_payload` and `AssignmentWrite` take it, satisfying the tests from 3.1
- [ ] 3.3 Replace `project_ref` / `project_name` on `CreateAssignmentInput`, `UpdateAssignmentInput` and `move_assignment` with `content`, trim the custom title and reject a blank one with a German error, and verify with `cargo test` including a blank-title rejection test
- [ ] 3.4 Write failing `slots.rs` tests: custom appointments share the window with project assignments in order-index order, and a re-slotted custom appointment keeps its title and marker; then make `plan_slot_updates` include `EventLink::Custom` so they pass
- [ ] 3.5 Write failing `protection.rs` tests: a custom appointment is not protected for update, delete or a day-changing move, and no project lookup runs for it, while a bare event stays protected; then update `is_protected_event` so they pass
- [ ] 3.6 Extend the Radicale write-path test with a custom appointment create, title update, move and delete, and verify it passes with `cargo test`

## 4. Generated bindings and call sites

- [ ] 4.1 Regenerate `src/generated/tauri.ts` by running the app in debug mode and verify it contains `AssignmentContent` and the `custom` event kind
- [ ] 4.2 Update `use-assignment-modal.ts`, `use-appointment-drag.ts` and `move-reconciliation-dialog.tsx` to pass `content`, and verify `bunx tsc --noEmit` and the existing `bun test` suites pass unchanged

## 5. Frontend cards

- [ ] 5.1 Write failing `bun test`s: `toCellEvent` maps `custom` with null project fields and the muted strip, `highlightKey` returns null for custom, and `cell-order.ts` orders custom and assignment cards together by order index
- [ ] 5.2 Add `"custom"` to `CellEvent.kind`, add an `isPlannerManaged` helper and use it in `cell-order.ts` and `withDropPreview`, satisfying the tests from 5.1
- [ ] 5.3 Write failing `timetable-cell.spec.tsx` tests: a custom card shows its title and times, is clickable and draggable, and shows no project status, warning icon, Daylite link or highlight toggle; then render custom events through the draggable card in `timetable-cell.tsx` so they pass

## 6. Modal

- [ ] 6.1 Write failing `assignment-modal.spec.tsx` tests: the type switch defaults to "Projekt", "Eigener Termin" hides the picker and shows a focused "Titel" field, a blank title disables save, and saving creates a custom appointment with the trimmed title
- [ ] 6.2 Add the type and title to the modal state in `use-assignment-modal.ts` and render the DaisyUI type switch and title input in `assignment-modal.tsx`, satisfying the tests from 6.1
- [ ] 6.3 Write failing tests for edit mode: a custom appointment opens with "Eigener Termin" and its title, changing the title and converting in both directions save under the same UID, an entered title triggers the unsaved-changes confirmation, no fixed-appointment notice appears for a custom appointment, and the type switch is disabled for a locked protected assignment; then implement them
- [ ] 6.4 Write a failing `next-day-quick-add.spec.ts` test that a custom create produces no ghost, then satisfy it

## 7. Integration checks

- [ ] 7.1 Run `cargo test --manifest-path src-tauri/Cargo.toml`, `bun test`, `bun lint`, `bun format:check` and `bunx tsc --noEmit` and verify all pass
- [ ] 7.2 In the running app, create a custom appointment next to a project assignment, verify both share the day's slots, rename it, drag it to another day and another employee, reorder it, convert it to a project and back, and delete it
- [ ] 7.3 Update the "Implemented" list in `README.md` with custom appointments and verify the line reads correctly

## Workflow follow-up

- Archive the change with `/opsx:archive` after review.
