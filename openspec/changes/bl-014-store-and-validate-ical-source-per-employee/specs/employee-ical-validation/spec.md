## ADDED Requirements

### Requirement: URL format validation
The system SHALL detect invalid URL format before making network calls.

#### Scenario: Invalid URL format
- **WHEN** user provides URL that is not a valid HTTP/HTTPS URL
- **THEN** validation fails immediately
- **AND** German error message explains the format issue

### Requirement: Independent iCal source testing
The system SHALL allow testing primary and absence iCal sources independently.

#### Scenario: Test primary assignment iCal
- **WHEN** user triggers test for primary iCal URL
- **THEN** only the primary URL is tested
- **AND** result is independent of absence iCal state

#### Scenario: Test absence iCal
- **WHEN** user triggers test for absence iCal URL
- **THEN** only the absence URL is tested
- **AND** result is independent of primary iCal state

### Requirement: German feedback with actionable hints
The system SHALL show German feedback with actionable error hints.

#### Scenario: Successful connection
- **WHEN** iCal URL is accessible and returns valid data
- **THEN** success message in German is shown
- **AND** includes timestamp of test

#### Scenario: Failed connection with actionable hint
- **WHEN** iCal URL test fails
- **THEN** German error message with specific hint is shown
- **AND** hint helps user understand how to fix the issue

### Requirement: Test timestamp persistence
The system SHALL persist the latest validation/test timestamp.

#### Scenario: Store test timestamp
- **WHEN** iCal test completes (success or failure)
- **THEN** timestamp is stored
- **AND** UI can display "Zuletzt getestet: [timestamp]"

### Requirement: Non-blocking validation
The system SHALL NOT block normal planning usage when tests fail.

#### Scenario: Failed test does not block planning
- **WHEN** iCal validation fails
- **THEN** user can still use the application
- **AND** planning continues with available data