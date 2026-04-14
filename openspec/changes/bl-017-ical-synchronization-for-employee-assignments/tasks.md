## 1. CalDAV Write Infrastructure (Rust)

- [ ] 1.1 Implement CalDAV PUT for VEVENT creation and update
- [ ] 1.2 Implement CalDAV DELETE for VEVENT removal
- [ ] 1.3 Add UUID generation for new assignments
- [ ] 1.4 Build VEVENT from assignment data using BL-015 encoding format (SUMMARY, DESCRIPTION, UID)
- [ ] 1.5 Apply BL-034 slot algorithm for DTSTART/DTEND based on assignment count per day

## 2. Tauri Commands

- [ ] 2.1 Add `create_assignment` command (employee reference, project reference, day → CalDAV PUT)
- [ ] 2.2 Add `update_assignment` command (UID, updated fields → CalDAV PUT with ETag)
- [ ] 2.3 Add `delete_assignment` command (UID, employee reference → CalDAV DELETE)

## 3. Safety Guards

- [ ] 3.1 Verify employee has a primary calendar URL configured before any write
- [ ] 3.2 Reject writes where target URL matches the employee's absence calendar URL
- [ ] 3.3 Add structured logging for all CalDAV write operations

## 4. Testing

- [ ] 4.1 Command tests: create/update/delete produce correct VEVENT format
- [ ] 4.2 Idempotency tests: repeated PUT with same UID produces one event (no duplicate)
- [ ] 4.3 Tests confirming absence calendar URLs are never written to
- [ ] 4.4 Error handling tests: CalDAV unavailable, auth failure, ETag conflict
