## 1. Project Status Detection

- [ ] 1.1 Implement `getProjectStatus(planradarProjectId)` function
- [ ] 1.2 Call Planradar API to fetch project details
- [ ] 1.3 Parse status field (active/archived/closed)
- [ ] 1.4 Handle API errors gracefully

## 2. Reactivation Service

- [ ] 2.1 Implement `reactivateProject(planradarProjectId)` function
- [ ] 2.2 Check current status before reactivation
- [ ] 2.3 Call Planradar API to reactivate if archived
- [ ] 2.4 Return success/error with appropriate status

## 3. Idempotency

- [ ] 3.1 Return success for already active projects (no API call)
- [ ] 3.2 Handle not-found case with clear error
- [ ] 3.3 Verify status after reactivation

## 4. Logging

- [ ] 4.1 Log reactivation attempt with project details
- [ ] 4.2 Log success with timestamp
- [ ] 4.3 Log failure with error message and stack trace
- [ ] 4.4 Include project name in all log entries

## 5. Testing

- [ ] 5.1 Write service tests for archived project scenario
- [ ] 5.2 Write service tests for active project scenario
- [ ] 5.3 Write service tests for not-found scenario
- [ ] 5.4 Write idempotency tests for already active projects
- [ ] 5.5 Write logging tests for success path
- [ ] 5.6 Write logging tests for failure path