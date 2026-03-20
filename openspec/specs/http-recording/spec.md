## ADDED Requirements

### Requirement: Record mode
The system SHALL record HTTP interactions when VCR_MODE=record.

#### Scenario: Live request recorded
- **GIVEN** VCR_MODE=record
- **WHEN** making an HTTP request
- **THEN** the request is made to live API
- **AND** response is saved to cassette file

#### Scenario: Cassette file created
- **GIVEN** HTTP interaction recorded
- **WHEN** recording completes
- **THEN** a cassette file exists in tests/cassettes/

### Requirement: Replay mode
The system SHALL replay recorded cassettes without network calls.

#### Scenario: Replay from cassette
- **GIVEN** VCR_MODE=replay (default)
- **AND** cassette exists for request
- **WHEN** making HTTP request
- **THEN** no network call is made
- **AND** response from cassette is returned

#### Scenario: Deterministic replay
- **GIVEN** test runs twice in replay mode
- **WHEN** executing test
- **THEN** results are identical both times
- **AND** network latency is zero

### Requirement: Header sanitization
The system SHALL remove auth headers before recording.

#### Scenario: Authorization header stripped
- **GIVEN** request with Authorization header
- **WHEN** recording in VCR_MODE=record
- **THEN** cassette file does not contain Authorization header

#### Scenario: API key header stripped
- **GIVEN** request with x-api-key header
- **WHEN** recording in VCR_MODE=record
- **THEN** cassette file does not contain x-api-key header

### Requirement: Git-crypt encryption
The system SHALL encrypt cassette files at rest.

#### Scenario: Cassettes encrypted
- **GIVEN** cassette files in tests/cassettes/
- **WHEN** checking git status
- **THEN** files are shown as encrypted (binary)

#### Scenario: CI unlocks cassettes
- **GIVEN** CI pipeline runs
- **WHEN** git-crypt unlock is executed
- **THEN** cassettes are readable for test execution