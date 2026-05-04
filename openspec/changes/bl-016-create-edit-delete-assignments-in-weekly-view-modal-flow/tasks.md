## 1. Resource URL Capture (Rust)

- [x] 1.1 Write failing test: `parse_caldav_report` returns href alongside each event
- [x] 1.2 Add href field to `RawVEvent` and `CalendarCellEvent` structs
- [x] 1.3 Update `parse_caldav_report` to extract `d:href` from each REPORT item
- [x] 1.4 Regenerate TypeScript bindings to include href in `CalendarCellEvent`

## 2. CalDAV Write Commands (Rust, TDD)

- [x] 2.1 Write failing unit test for iCal VCALENDAR payload builder (08:00–16:00 window)
- [x] 2.2 Implement iCal payload builder
- [x] 2.3 Write failing VCR test for `create_assignment` command
- [x] 2.4 Implement `create_assignment` Tauri command (CalDAV PUT to `{calendar_url}/{uid}.ics`)
- [x] 2.5 Write failing VCR test for `update_assignment` command
- [x] 2.6 Implement `update_assignment` Tauri command (CalDAV PUT to stored href)
- [x] 2.7 Write failing VCR test for `delete_assignment` command
- [x] 2.8 Implement `delete_assignment` Tauri command (CalDAV DELETE to stored href)

## 3. Project Picker Service (Frontend, TDD)

- [x] 3.1 Write failing service test: returns only `new_status` and `in_progress` projects via BL-022
- [x] 3.2 Implement project picker service using BL-022 `daylite-project-query`

## 4. AssignmentModal Component (Frontend, TDD)

- [x] 4.1 Write failing render test: modal in create mode shows empty project picker and save button
- [x] 4.2 Write failing render test: modal in edit mode shows pre-populated project and delete button
- [x] 4.3 Write failing render test: delete confirmation dialog renders correctly
- [x] 4.4 Implement `AssignmentModal` component (DaisyUI modal, create/edit/delete flows)
- [x] 4.5 Connect modal save/delete actions to Tauri commands

## 5. Cell Wiring and Grid Integration (Frontend, TDD)

- [ ] 5.1 Write failing render test: empty cell renders a clickable add affordance
- [ ] 5.2 Write failing render test: assigned cell renders as clickable with assignment data
- [ ] 5.3 Wire cell click handlers to open modal (empty → create mode, assigned → edit mode)
- [ ] 5.4 Implement reload after save via `reloadAssignments()`
- [ ] 5.5 Show German error message in modal on save/delete failure

## 6. Edge Cases (TDD)

- [ ] 6.1 Write failing test: unsaved changes dialog renders when closing modal with dirty state
- [ ] 6.2 Implement unsaved changes confirmation dialog
- [ ] 6.3 Escape key closes modal (triggers unsaved changes dialog if dirty)
- [ ] 6.4 Click outside modal closes modal (triggers unsaved changes dialog if dirty)
