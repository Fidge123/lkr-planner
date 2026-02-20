# BL-022: Project Search Outside Standard-Filter (Assignment Modal)

## Scope
- Provide project search in the assignment modal that can also find projects outside the Standard-Filter result set.
- Restrict search result set to projects with status `new_status` or `in_progress`.
- Search by at least project name + external reference (if available).
- Support result limiting for UI usage (`limit=5` for modal result list).

## Acceptance Criteria
- User can find and assign a project even if it is not in the Standard-Filter result.
- Search returns only projects with status `new_status` or `in_progress`.
- Search API supports deterministic first 5 results for modal display.

## Tests (write first)
- Service tests for search API filtering by status and result limit behavior.
- Service tests for result ranking and error cases.
- UI tests for search input, result list, and selection in the assignment modal.
