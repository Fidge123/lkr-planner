## 1. Link Detection (TDD)

- [ ] 1.1 (red) Service tests: linked project returns the ID, unlinked returns none, unset field handled
- [ ] 1.2 (green) Implement link check and read the ID from the `planradar-link` custom field

## 2. Link field provisioning and write (TDD)

- [ ] 2.1 (red) Test the `planradar-link` field is created when missing and reused when present
- [ ] 2.2 (green) Implement field provisioning
- [ ] 2.3 (red) Test writing a selected Planradar project ID, idempotent re-link (no duplicate write), and that link create and failure emit sync events
- [ ] 2.4 (green) Implement link write with check-before-write and sync event logging

## 3. Project Title Fallback (folded in from BL-036) (TDD)

- [ ] 3.1 (red) Title-fallback tests: Planradar name, then single Daylite company, then Daylite project name, plus the empty custom-name slot
- [ ] 3.2 (green) Implement title resolution and the single-linked-company helper
- [ ] 3.3 Wire the resolved title into the assignment card (rendering already exists in assignment-persistence / timetable-cell)
