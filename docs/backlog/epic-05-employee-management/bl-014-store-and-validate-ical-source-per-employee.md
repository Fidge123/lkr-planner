# BL-014: Store and Validate iCal Source per Employee

## Scope
- Save two iCal URLs per employee (from Daylite):
  - Primary assignment iCal
  - Secondary absence iCal (vacation/sick leave)
- Basic validation + connection test (manually triggerable).

## Acceptance Criteria
- Invalid URLs are handled cleanly.
- Connection test provides clear success/error message.
- Absence iCal can be validated and tested independently from primary iCal.

## Tests (write first)
- Parser/validation tests.
- Error case tests for unreachable calendar sources.
- Tests for separate validation/reporting of primary vs absence iCal.
