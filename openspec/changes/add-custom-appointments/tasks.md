# Tasks

## 1. Architecture record

- [ ] 1.1 Write ADR 0015 in `docs/adr` recording the `lkr-planner:custom` DESCRIPTION marker, the classification order and the rejected alternatives, and verify it follows the format of the existing ADRs

## 2. Backend classification

- [ ] 2.1 Write failing `cargo test`s in `events/classify.rs`: a first line of `lkr-planner:custom` classifies as custom, also with surrounding whitespace, BOM or zero-width space and with further lines after it; the marker on a later line, a prefix like `lkr-planner:customer` and any other first line stay bare; a `daylite:` reference stays a project assignment
- [ ] 2.2 Replace `PendingEvent.project_ref` with an `EventLink` enum (`Project`, `Custom`, `Bare`), add `CalendarEventKind::Custom`, map custom events in `resolve_event` with no project status, category or reference, and verify the tests from 2.1 and the existing `events` tests pass
- [ ] 2.3 Write a failing test that `project_references` skips custom events so no Daylite lookup is made for them, then satisfy it

## 3. Backend writes

- [ ] 3.1 Write failing `ical.rs` tests: a custom payload carries the title as SUMMARY and `lkr-planner:custom` as DESCRIPTION, and round-trips a title with commas, semicolons, backslashes and umlauts unchanged
- [ ] 3.2 Introduce `AssignmentContent` (`project` / `custom`) and make `build_ical_payload` and `AssignmentWrite` take it, satisfying the tests from 3.1
- [ ] 3.3 Replace `project_ref` / `project_name` on `CreateAssignmentInput`, `UpdateAssignmentInput` and `move_assignment` with `content`, trim the custom title and reject a blank one with a German error, and verify with `cargo test` including a blank-title rejection test
- [ ] 3.4 Write failing `slots.rs` tests: custom appointments share the window with project assignments in order-index order, and a re-slotted custom appointment keeps its title and DESCRIPTION; then make `plan_slot_updates` include `EventLink::Custom` so they pass
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

- [ ] 6.1 Write failing `assignment-modal.spec.tsx` tests for create mode: the type switch defaults to "Projekt", "Eigener Termin" hides the picker and shows a focused "Titel" field, a blank title disables save, an entered title triggers the unsaved-changes confirmation, and saving creates a custom appointment with the trimmed title
- [ ] 6.2 Add the type and title to the modal state in `use-assignment-modal.ts` and render the DaisyUI type switch and title input in `assignment-modal.tsx`, satisfying the tests from 6.1
- [ ] 6.3 Write failing tests for edit mode: no type switch is shown for either kind, a custom appointment opens with its title and no picker, changing the title saves under the same UID, clearing it disables save, and no fixed-appointment notice appears; then implement them

## 7. Next-day ghost

- [ ] 7.1 Write failing `next-day-quick-add.spec.ts` tests: a custom create produces a ghost carrying its title on the next visible day, and the existing project cases still hold
- [ ] 7.2 Switch `GhostSuggestion` and the `create` variant of `ModalSaveAction` to `AssignmentContent`, satisfying the tests from 7.1
- [ ] 7.3 Write failing `timetable-cell.spec.tsx` tests that a custom ghost shows its title and clicking it creates a custom appointment that chains to the following day, then wire the ghost click to pass the stored content

## 8. Integration checks

- [ ] 8.1 Run `cargo test --manifest-path src-tauri/Cargo.toml`, `bun test`, `bun lint`, `bun format:check` and `bunx tsc --noEmit` and verify all pass
- [ ] 8.2 In the running app, create a custom appointment next to a project assignment, verify both share the day's slots, chain it to the next days via the ghost, rename it, drag it to another day and another employee, reorder it and delete it; verify in ZEP that its DESCRIPTION reads `lkr-planner:custom`
- [ ] 8.3 Update the "Implemented" list in `README.md` with custom appointments and verify the line reads correctly

## Workflow follow-up

- Archive the change with `/opsx:archive` after review.
