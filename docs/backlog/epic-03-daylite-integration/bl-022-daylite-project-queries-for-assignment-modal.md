# BL-022: Daylite Project Queries for Assignment Modal

## Scope
- Provide Daylite project query capabilities used by the assignment modal.
- Support text search by at least project name and external reference (if available).
- Restrict search result set to projects with status `new_status` or `in_progress`.
- Support deterministic result limiting (`limit=5`) for modal usage.
- Provide an explicit query mode for overdue projects (`Überfällig`) used by default suggestions.

## Acceptance Criteria
- Query API returns only `new_status` and `in_progress` projects for text search.
- Query API returns deterministic first 5 results for identical input.
- Overdue query returns deterministic top 5 overdue projects for suggestion usage.
- API failures are normalized into German user-facing error messages.

## Dependencies
- Depends on existing Daylite project read/search command foundation (BL-006).

## Out of Scope
- Modal UI behavior and suggestion ordering logic.

## Tests (write first)
- Service tests for search filtering, ordering, and limit behavior.
- Service tests for overdue query behavior.
- Error handling tests for timeout and malformed payloads.
