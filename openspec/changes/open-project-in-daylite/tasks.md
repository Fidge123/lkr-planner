## 1. Verify the deep link against a real Daylite

- [ ] 1.1 Pick a project visible in both the Daylite macOS app and the Daylite API, and note the numeric id from its `/v1/projects/<id>` reference
- [ ] 1.2 Open `daylite4://ShowObject/Project/<id>` from the browser address bar or `open` in a terminal and confirm Daylite comes forward on that exact project
- [ ] 1.3 Record the result in design.md; if the id does not address the project, stop here and reopen the design rather than continuing

## 2. Deep Link Command (TDD)

- [ ] 2.1 (red) Rust unit test: `/v1/projects/2035` translates to `daylite4://ShowObject/Project/2035`
- [ ] 2.2 (red) Rust unit test: a reference that is not `/v1/projects/<numeric id>` yields a German error instead of a URL
- [ ] 2.3 (green) Implement the reference-to-URL translation as a pure function in `src-tauri/src/integrations/daylite/`
- [ ] 2.4 (green) Add the `daylite_open_project` command wrapping the translation and `app.opener().open_url(url, None::<&str>)`
- [ ] 2.5 Register the command in `specta_builder` in `src-tauri/src/lib.rs` and regenerate `src/generated/tauri.ts`
- [ ] 2.6 Confirm `src-tauri/capabilities/default.json` needs no change by opening a card's project from a dev build

## 3. Frontend Service (TDD)

- [ ] 3.1 (red) Service test: opening a project reference invokes the command with that reference
- [ ] 3.2 (red) Service test: a backend error surfaces as a German message rather than throwing
- [ ] 3.3 (green) Implement the service wrapper alongside the existing Daylite services

## 4. Card Action Area (TDD)

- [ ] 4.1 (red) Component test: an assignment card with a resolved project renders the action with its German accessible name
- [ ] 4.2 (red) Component test: an unresolved assignment, a bare event, and an absence render no action
- [ ] 4.3 (red) Component test: the action is not rendered on the card currently being dragged
- [ ] 4.4 (green) Add the action area as a sibling of the card button inside the existing `<li>`, positioned over the card's right edge
- [ ] 4.5 (green) Add right padding on the card button so the title stops before the action area
- [ ] 4.6 (green) Wire the action to the service with the card's `projectRef`

## 5. Interaction and Appearance

- [ ] 5.1 In a dev build, confirm pressing the action opens Daylite and does not open the edit modal
- [ ] 5.2 In a dev build, confirm pressing and moving on the action starts no drag, and that dragging the card elsewhere still works
- [ ] 5.3 Check the narrowest column the grid produces: a long title wraps and does not run under the icon
- [ ] 5.4 Confirm the action carries no border and no background, and matches the card's hover feedback

## 6. Close Out

- [ ] 6.1 Run `bun test` and `bun run lint`
- [ ] 6.2 Run `bunx openspec validate open-project-in-daylite`
