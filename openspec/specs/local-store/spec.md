## Requirements

### Requirement: Restart-safe Local Configuration
The system SHALL store application configuration points in a local, persistent JSON store across restarts.

#### Scenario: Configuration persistence
- **GIVEN** the application is running
- **WHEN** the user or system updates configuration properties
- **THEN** the values are saved to a typed local JSON file in the application data directory
- **AND** the values are restored upon application restart

### Requirement: Error Reporting Payloads
The system SHALL provide structured error payloads consisting of user-facing German messages and technical details.

#### Scenario: Error UI display
- **GIVEN** a backend or domain error occurs
- **WHEN** the error propagates to the frontend
- **THEN** a user-facing error message in German is provided for display
- **AND** the technical details are logged via technical error fields
