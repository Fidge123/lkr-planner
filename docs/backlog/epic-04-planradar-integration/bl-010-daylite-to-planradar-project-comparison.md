# BL-010: Daylite -> Planradar Project Comparison

## Scope
- Comparison logic:
  - Does a corresponding Planradar project exist (via Daylite custom field link)?
  - If no link exists: User can link an existing Planradar project or create a new project via cloning.
  - If linked Planradar project is archived/closed: automatically reactivate (unarchive/reopen).
- Persist the linked Planradar project reference as a custom field in Daylite.
- Use configurable Daylite field mapping for this link:
  - Default field label: `Planradar-Projekt-ID`
  - Stored field value: `planradarProjectId` returned by Planradar API
  - Daylite field key/id is metadata to locate the field, not the stored project id itself
- Ensure idempotency.

## Acceptance Criteria
- Multiple runs do not create duplicates.
- Every action is logged as a sync event.
- After successful linking, the link is stored in Daylite and reused in the next run.

## Tests (write first)
- Service tests for cases: new, already existing, closed, API error.
- Test for persistence and reuse of the Daylite custom field link.
