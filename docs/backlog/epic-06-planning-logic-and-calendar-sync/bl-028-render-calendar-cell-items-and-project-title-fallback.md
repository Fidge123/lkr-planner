# BL-028: Render Calendar Cell Items and Project Title Fallback

## Scope
- Render the following item types in each calendar cell:
  - All-day items from absence calendar (read-only, title from source event).
  - Vacation/holiday items (read-only, title uses German holiday name).
  - Project assignments (editable, sorted by start time).
  - Preexisting appointments (read-only, show start time + appointment title).
- Define project item title fallback in this exact order:
  - Custom name
  - Planradar project name
  - Daylite company (only if exactly one company is linked)
  - Daylite project name (if no or multiple companies are linked)

## Acceptance Criteria
- Calendar cells show all required item types with correct read-only/editable behavior.
- Project items are sorted by start time.
- Preexisting appointments display start time and title.
- Project title generation follows the defined fallback order.

## Tests (write first)
- UI tests for rendering all item types in one cell.
- UI tests for read-only behavior of absence, vacation, and preexisting appointments.
- Unit tests for project title fallback order.
- UI tests for project sort order by start time.
