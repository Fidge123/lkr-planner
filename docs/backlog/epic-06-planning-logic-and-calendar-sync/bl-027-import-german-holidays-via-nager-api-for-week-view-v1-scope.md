# BL-027: Import German Holidays via Nager API for Week View (v1 Scope)

## Scope
- Integrate public holiday import from `https://date.nager.at` for Germany (`DE`).
- Fetch holidays per year from Nager API and map to app day model.
- In v1, only consider:
  - Germany-wide holidays (`global=true`)
  - Mecklenburg-Vorpommern (`MV`) holidays
- Exclude other state-specific holidays in v1.
- Cache yearly holiday data locally to avoid repeated API calls during week navigation.

## Acceptance Criteria
- Week view includes only Germany-wide and MV-relevant holidays.
- Holiday names are available in German for calendar item rendering.
- Weeks crossing year boundaries correctly show holidays from both years.
- API failure does not break planning view; user gets a German warning state.

## Tests (write first)
- Service tests for Nager API mapping/filtering (`DE`, `global`, `MV`).
- Tests that non-MV state-only holidays are excluded in v1.
- Tests for year-boundary fetch behavior (two-year query).
- Error-state test for Nager API timeout/failure.
