# BL-014: Validate and Test Employee iCal Sources

## Scope
- Provide validation tooling for employee primary assignment iCal and secondary absence iCal URLs.
- Add a manual connection test action per source URL.
- Show clear German feedback for success/failure including actionable hints.
- Persist latest validation/test timestamp for transparency in the UI.

## Acceptance Criteria
- Invalid URL format is detected before any network call.
- Connection tests can run independently for primary and absence iCal.
- Test result includes at least HTTP/access outcome and readable German status text.
- Failed tests do not block normal planning usage.

## Dependencies
- Builds on existing Daylite contact iCal read/write support (BL-008 completed).

## Out of Scope
- Employee CRUD against Daylite contacts.

## Tests (write first)
- Validation tests for allowed/disallowed URL formats.
- Service tests for independent primary vs absence test execution.
- UI tests for result rendering and retry behavior.
