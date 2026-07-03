## 1. Project Status Detection (TDD)

- [ ] 1.1 (red) Tests: status parsed as active / archived / closed, and API errors surface gracefully
- [ ] 1.2 (green) Implement `getProjectStatus` via the project read endpoint

## 2. Reactivation Service (TDD)

- [ ] 2.1 (red) Test reactivate calls archive_project with status 1 only when the project is archived
- [ ] 2.2 (green) Implement `reactivateProject`

## 3. Idempotency (TDD)

- [ ] 3.1 (red) Tests: already-active returns success with no API call; not-found returns a clear error
- [ ] 3.2 (green) Implement the status check before reactivation and not-found handling

## 4. Logging (TDD)

- [ ] 4.1 (red) Test reactivation success and failure each emit a sync event with project ID, name and timestamp
- [ ] 4.2 (green) Emit sync events on the reactivation paths
