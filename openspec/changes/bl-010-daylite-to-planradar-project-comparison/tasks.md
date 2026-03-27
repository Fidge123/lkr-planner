## 1. Link Detection

- [ ] 1.1 Implement check for existing Planradar link on Daylite project
- [ ] 1.2 Read Planradar project ID from Daylite custom field
- [ ] 1.3 Handle case where custom field is not set

## 2. Link Operations

- [ ] 2.1 Implement link creation with Planradar project selection
- [ ] 2.2 Write Planradar project ID to Daylite custom field
- [ ] 2.3 Implement idempotent link behavior (check before write)

## 3. Logging

- [ ] 3.1 Add sync event logging for link operations
- [ ] 3.2 Log successful links with timestamp
- [ ] 3.3 Log failed link attempts with error details

## 4. Testing

- [ ] 4.1 Service tests for linked/unlinked/project-not-found flows
- [ ] 4.2 Persistence tests for Daylite custom field writes
- [ ] 4.3 Idempotency tests for repeated link operations