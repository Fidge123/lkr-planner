# BL-010: Link Existing Planradar Projects to Daylite

## Scope
- Determine if Daylite project has a linked Planradar project reference.
- If no link exists, allow linking an already existing Planradar project.
- Persist Planradar project ID into configured Daylite custom field.
- Ensure idempotent link behavior across repeated runs.

## Acceptance Criteria
- Existing link is reused without duplicate link writes.
- New link writes persist Planradar ID in configured Daylite field.
- Link operation is logged as sync event.

## Dependencies
- Depends on BL-009 Planradar API client.

## Out of Scope
- Creating new Planradar projects.
- Reactivating archived Planradar projects.

## Tests (write first)
- Service tests for linked/unlinked/project-not-found flows.
- Persistence tests for Daylite custom field writes.
- Idempotency tests for repeated link operations.
