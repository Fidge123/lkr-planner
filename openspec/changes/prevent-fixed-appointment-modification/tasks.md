## 1. Extract reusable description parser

- [ ] 1.1 Write failing `cargo test`s for a standalone `parse_daylite_reference(description: &str) -> Option<String>` function covering: valid `daylite:/<path>` first line, no reference present, empty description
- [ ] 1.2 Extract the parsing logic out of `classify_event` (`src-tauri/src/integrations/calendar/events.rs`) into this function, satisfying the tests, and update `classify_event` to call it

## 2. Extend Daylite project lookup with category

- [ ] 2.1 Write failing `cargo test`s asserting `fetch_project_by_reference` returns `category` alongside name/status (protected category present, absent, `null`)
- [ ] 2.2 Extend `fetch_project_by_reference` in `src-tauri/src/integrations/daylite/projects.rs` to include `category` in its return value, satisfying the tests
- [ ] 2.3 Update the existing call site in `load_week_events` (`calendar/commands.rs`) to ignore the new field
- [ ] 2.4 Add `FIXED_APPOINTMENT_CATEGORY: &str = "Termin FIX geplant"` constant alongside `OVERDUE_CATEGORY`

## 3. Backend guard

- [ ] 3.1 Write failing `cargo test`s for a `is_protected_event(href) -> bool`-style guard: protected category, non-protected category, no project reference, project lookup failure (fail open)
- [ ] 3.2 Implement the guard in `src-tauri/src/integrations/calendar/commands.rs` (or a new module), fetching the event by `href`, parsing its Daylite reference, and checking the project's category, satisfying the tests
- [ ] 3.3 Wire the guard into `update_assignment`: reject with a German error before the CalDAV PUT if the event is protected
- [ ] 3.4 Wire the guard into `delete_assignment`: reject with a German error before the CalDAV DELETE if the event is protected
- [ ] 3.5 Add `cargo test` coverage confirming `create_assignment` is unaffected by the guard

## 4. Frontend disabled state

- [ ] 4.1 Write failing `bun test`s for the assignment modal: save/delete disabled and German notice shown when the loaded assignment's project category is `"Termin FIX geplant"`
- [ ] 4.2 Update `src/app/components/assignment-modal.tsx` to look up the project's category from the Daylite project cache and disable save/delete with a notice, satisfying the tests
- [ ] 4.3 Add `bun test` coverage for surfacing the backend's German rejection message if a stale edit/delete is submitted anyway

## 5. Verification

- [ ] 5.1 Run `cargo test` and confirm all new and existing tests pass
- [ ] 5.2 Run `bun test` and confirm all new and existing tests pass
- [ ] 5.3 Run `bun lint` and fix any issues
- [ ] 5.4 Manually verify in the running app: an assignment linked to a "Termin FIX geplant" project shows disabled edit/delete controls with a German notice, and a direct backend call to update/delete it is rejected
