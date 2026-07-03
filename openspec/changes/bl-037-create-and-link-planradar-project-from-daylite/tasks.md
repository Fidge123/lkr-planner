## 1. Project Creation Service (TDD)

- [ ] 1.1 (red) Service test: blank create from a Daylite project returns the new Planradar ID with the Daylite name
- [ ] 1.2 (green) Implement blank create via the create-project endpoint
- [ ] 1.3 (red) Service test: copy from a source project passes the chosen name and aspect toggles, then applies the edits
- [ ] 1.4 (green) Implement copy-then-edit creation via copy_project plus project update

## 2. Idempotency Logic (TDD)

- [ ] 2.1 (red) Tests: existing valid link returns the existing project (no create); orphan link logs a sync issue and creates new
- [ ] 2.2 (green) Implement idempotent create (read `planradar-link`, verify the project exists, branch accordingly)

## 3. Link Persistence (TDD)

- [ ] 3.1 (red) Test the Planradar ID is written to `planradar-link` after creation; write failure queues a retry and logs a sync issue
- [ ] 3.2 (green) Implement write-back with retry queue and local mapping backup

## 4. Source Project Selection UI

- [ ] 4.1 Add "Create in Planradar" action to unlinked projects
- [ ] 4.2 Show source project list with search/filter and the aspect toggles
- [ ] 4.3 Open the edit form for the copied project before finishing
- [ ] 4.4 Handle the blank create option
