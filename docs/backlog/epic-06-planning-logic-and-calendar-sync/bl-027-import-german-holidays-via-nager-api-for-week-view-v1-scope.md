# BL-027: Import German Holidays via Nager API for Week View (v1 Scope)

## Scope
- Integrate public holiday import from `https://date.nager.at` for Germany (`DE`).
- Fetch holiday data per year and map to week-view day model.
- In v1 include only:
  - Germany-wide holidays (`global=true`)
  - Mecklenburg-Vorpommern (`MV`) holidays
- Exclude other state-specific holidays in v1.
- Cache year data locally to avoid unnecessary repeated API calls.

## Acceptance Criteria
- Week view includes only Germany-wide and MV-relevant holidays.
- Holiday names are available for German UI rendering.
- Year-boundary weeks resolve holidays from both years.
- API failure shows a German warning state without breaking planning.

## Dependencies
- Provides holiday inputs for calendar-cell item composition (BL-035).

## Out of Scope
- Additional Bundesland-specific holiday sets beyond MV.

## Tests (write first)
- Service tests for Nager API mapping/filtering (`DE`, `global`, `MV`).
- Tests that non-MV state-only holidays are excluded.
- Year-boundary fetch tests.
- Error-state tests for timeout/failure behavior.
