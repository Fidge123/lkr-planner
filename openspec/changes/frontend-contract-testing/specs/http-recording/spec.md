## ADDED Requirements

### Requirement: Shared record/replay transport across integrations
The system SHALL provide a single reusable record/replay HTTP transport, keyed on method, path, query, and body, that the Daylite, holiday (Nager), and CalDAV/ZEP clients all route through.
The transport SHALL remain test-only (`#[cfg(test)]`) so production builds are unaffected.
Adding a new recorded integration SHALL mean routing its requests through this transport, not duplicating the record/replay logic.

#### Scenario: Holidays replays from a cassette
- **GIVEN** VCR_MODE=replay and a committed holidays cassette
- **WHEN** the holidays client fetches public holidays
- **THEN** the response is served from the cassette with no network call

#### Scenario: CalDAV replays from a cassette
- **GIVEN** VCR_MODE=replay and a committed ZEP/CalDAV cassette
- **WHEN** the CalDAV client fetches week events
- **THEN** the response is served from the cassette with no network call

#### Scenario: Daylite recording is unchanged
- **WHEN** the Daylite client records or replays via the shared transport
- **THEN** the existing Daylite cassettes and tests still pass
