# BL-036: Calendar Cell Rendering and Project Title Fallback

## Scope
- Render composed cell items from BL-035 with correct read-only/editable behavior.
- Apply project title fallback rule in exact order:
  - custom name
  - Planradar project name
  - Daylite company (only if exactly one linked)
  - Daylite project name (if none or multiple linked companies)
- Render preexisting appointments with start/end time + title text.

## Acceptance Criteria
- Cell UI renders all item types with correct interaction constraints.
- Project item titles always follow fallback order.
- Every item displays start time, end time and title.

## Dependencies
- Depends on BL-035 composed model availability.

## Tests (write first)
- UI tests for all item types in one cell.
- Unit tests for project title fallback order.
- UI tests for read-only enforcement on absence/holiday/appointment rows.
