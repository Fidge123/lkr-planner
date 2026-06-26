## 1. CalDAV Write Infrastructure (Rust)

- [x] 1.1 Implement CalDAV PUT for VEVENT creation and update
- [x] 1.2 Implement CalDAV DELETE for VEVENT removal
- [x] 1.3 Add UUID generation for new assignments
- [x] 1.4 Build VEVENT from assignment data using BL-015 encoding format (SUMMARY, DESCRIPTION, UID)
- [x] 1.5 Use a fixed 08:00-16:00 DTSTART/DTEND window per assignment (per-day slot redistribution for multiple same-day assignments is tracked separately in BL-034)

## 2. Tauri Commands

- [x] 2.1 Add `create_assignment` command (employee reference, project reference, day → CalDAV PUT)
- [x] 2.2 Add `update_assignment` command (href, UID, updated fields → CalDAV PUT, last-write-wins; ETag conflict detection dropped as YAGNI)
- [x] 2.3 Add `delete_assignment` command (href → CalDAV DELETE, idempotent on already-absent event)

## 3. Safety Guards

- [x] 3.1 Verify employee has a primary calendar URL configured before any write
- [x] 3.2 Reject writes where target URL matches any configured absence calendar URL
- [x] 3.3 Add structured logging for all CalDAV write operations

## 4. Testing

- [x] 4.1 Command tests: create/update/delete produce correct VEVENT format (VCR tests against live CalDAV)
- [x] 4.2 Idempotency tests: repeated PUT with same UID produces one event (stable `{uid}.ics` resource)
- [x] 4.3 Tests confirming absence calendar URLs are never written to (`targets_absence_calendar`)
- [x] 4.4 Error handling tests: CalDAV unavailable, auth failure, non-2xx status
