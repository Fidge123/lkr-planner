# BL-015: Switch Planning Table from Dummy Data to Real Data Source

## Scope
- Decouple `dummy-data`, connect service layer instead.
- Add load, empty, and error states in weekly view.

## Acceptance Criteria
- Weekly view works with persistent data.
- Error states are understandable for users (German).

## Tests (write first)
- UI tests for Loading/Empty/Error.
