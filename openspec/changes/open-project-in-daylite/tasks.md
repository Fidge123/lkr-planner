## 1. Confirm the deep link against a real Daylite

- [x] 1.1 Pick a project visible in both the Daylite macOS app and the Daylite API, and note the numeric id from its `/v1/projects/<id>` reference
- [x] 1.2 Open `daylite://Command=ShowObject&Entity=Project&ID=<id>` with that id and confirm Daylite comes forward on that same project
- [x] 1.3 Record the confirmed URL form in design.md

## 2. Deep Link Command (TDD)

- [x] 2.1 (red) Rust unit test: `/v1/projects/2035` translates to `daylite://Command=ShowObject&Entity=Project&ID=2035`
- [x] 2.2 (red) Rust unit test: a reference that is not `/v1/projects/<numeric id>` yields a German error instead of a URL
- [x] 2.3 (green) Implement the reference-to-URL translation as a pure function in `src-tauri/src/integrations/daylite/`
- [x] 2.4 (green) Add the `daylite_open_project` command wrapping the translation and `app.opener().open_url(url, None::<&str>)`
- [x] 2.5 Register the command in `specta_builder` in `src-tauri/src/lib.rs` and regenerate `src/generated/tauri.ts`

## 3. Frontend Service (TDD)

- [x] 3.1 (red) Service test: opening a project reference invokes the command with that reference
- [x] 3.2 (red) Service test: a backend error surfaces as a German message rather than throwing
- [x] 3.3 (green) Implement the service wrapper alongside the existing Daylite services

## 4. Card Loses Its Click (TDD)

- [x] 4.1 (red) Component test: an assignment card renders no control of its own around its times and title
- [x] 4.2 (red) Component test: an assignment card renders an edit action with its German accessible name, including when the project is unresolved
- [x] 4.3 (green) Turn the card from a button into a container laying out body, edit action, and a slot for further actions
- [x] 4.4 (green) Move `onEventClick` from the card to the edit action
- [x] 4.5 (green) Move the drag `listeners` and `setActivatorNodeRef` to the card body, keep `setNodeRef` on the container, and stop spreading `attributes`

## 5. Daylite Action (TDD)

- [x] 5.1 (red) Component test: a resolved assignment renders the Daylite action after the edit action, with its German accessible name
- [x] 5.2 (red) Component test: an unresolved assignment, a bare event, and an absence render no Daylite action
- [x] 5.3 (red) Component test: neither action is rendered on the card currently being dragged
- [x] 5.4 (green) Add the Daylite action to the action area and wire it to the service with the card's `projectRef`

## 6. Existing Tests and Callers

- [x] 6.1 Update `timetable-cell.spec.tsx` and any grid-level spec that opens the edit modal by clicking an assignment card
- [x] 6.2 Confirm `timetable-row.tsx` still supplies the edit callback and needs no signature change

## 7. Interaction and Appearance

- [ ] 7.1 In a dev build, confirm the edit action opens the modal and the Daylite action opens Daylite, neither triggering the other
- [ ] 7.2 In a dev build, confirm pressing and moving on either action starts no drag, and that dragging the card by its body still reschedules and reassigns as before
- [ ] 7.3 Confirm clicking the card body does nothing
- [ ] 7.4 Check the narrowest column the grid produces: a long title wraps, the two icons stay on the right edge, and neither overlaps the title
- [ ] 7.5 Confirm both actions carry no border and no background, and are distinguishable on hover from the card's own hover indicator

## 8. Close Out

- [x] 8.1 Run `bun test` and `bun run lint`
- [x] 8.2 Run `bunx openspec validate open-project-in-daylite`
