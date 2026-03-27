## 1. Sync Infrastructure

- [ ] 1.1 Add UUID generation for assignments
- [ ] 1.2 Create iCal event builder with proper formatting
- [ ] 1.3 Implement iCal push/update operations

## 2. Sync Orchestration

- [ ] 2.1 Trigger sync on assignment create
- [ ] 2.2 Trigger sync on assignment update
- [ ] 2.3 Trigger sync on assignment delete
- [ ] 2.4 Implement idempotent sync logic (use stable UID)

## 3. Status Tracking

- [ ] 3.1 Add sync status field to assignment model
- [ ] 3.2 Track last sync timestamp
- [ ] 3.3 Expose status for troubleshooting UI
- [ ] 3.4 Handle failed syncs with retry queue

## 4. Safety Guards

- [ ] 4.1 Verify primary iCal URL exists before sync
- [ ] 4.2 Never construct absence calendar URL for writing
- [ ] 4.3 Add logging for all sync operations

## 5. Testing

- [ ] 5.1 Sync service tests for create/update/delete and retry behavior
- [ ] 5.2 Idempotency tests across repeated sync executions
- [ ] 5.3 Tests ensuring absence calendars are never written