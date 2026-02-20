# BL-037: Create and Link Planradar Project from Daylite

## Scope
- Create new Planradar project when no link exists and user chooses create path.
- Select template/source from list of existing projects (allow user to filter projects to select right template project).
- Persist created Planradar project ID back to Daylite custom field.
- Keep operation idempotent for repeated executions.

## Acceptance Criteria
- Creation path produces one new Planradar project per eligible unlinked Daylite project.
- Created Planradar ID is persisted and reused on next run.
- Missing mapping produces sync issue instead of hard crash.

## Dependencies
- Depends on BL-009.

## Out of Scope
- Reactivating already linked archived projects.

## Tests (write first)
- Service tests for create success, mapping miss, and API failure.
- Idempotency tests preventing duplicate Planradar project creation.
- Persistence tests for Daylite link writeback.
