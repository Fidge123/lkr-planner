## 1. Project Creation Service

- [ ] 1.1 Implement `createPlanradarProject(dayliteProject, templateId?)` function
- [ ] 1.2 Call Planradar API to create project
- [ ] 1.3 Map Daylite project name to Planradar project
- [ ] 1.4 Copy template properties if template selected

## 2. Idempotency Logic

- [ ] 2.1 Read existing Planradar ID from Daylite custom field
- [ ] 2.2 Check if Planradar project exists with that ID
- [ ] 2.3 Return existing if valid, create new if not
- [ ] 2.4 Log sync issue for orphan Planradar IDs

## 3. Link Persistence

- [ ] 3.1 Write Planradar ID to Daylite custom field after creation
- [ ] 3.2 Implement retry queue for failed writes
- [ ] 3.3 Store local backup of mapping

## 4. Template Selection UI

- [ ] 4.1 Add "Create in Planradar" button to unlinked projects
- [ ] 4.2 Show template project list modal
- [ ] 4.3 Implement search/filter for templates
- [ ] 4.4 Handle blank template option

## 5. Testing

- [ ] 5.1 Write service tests for successful creation
- [ ] 5.2 Write service tests for mapping miss
- [ ] 5.3 Write service tests for API failure
- [ ] 5.4 Write idempotency tests for duplicate prevention
- [ ] 5.5 Write persistence tests for Daylite writeback