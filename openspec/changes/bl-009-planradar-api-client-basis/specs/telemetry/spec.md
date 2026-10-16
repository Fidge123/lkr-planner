## MODIFIED Requirements

### Requirement: Error events
The system SHALL record an error event when a backend operation fails, when an outbound request to an integration fails, and when the user interface encounters an uncaught error.

#### Scenario: Backend command fails
- **GIVEN** telemetry is enabled
- **WHEN** a backend command returns a structured error
- **THEN** an error event is recorded carrying the operation name, the integration name, the error code, and a sanitized technical message

#### Scenario: Outbound integration request fails
- **GIVEN** telemetry is enabled
- **WHEN** a request to Daylite, Planradar, the ZEP CalDAV server, or the holiday API fails or returns an unsuccessful status
- **THEN** an error event is recorded carrying the integration name, the request kind, and the response status when one was received

#### Scenario: Retried request records every attempt
- **GIVEN** telemetry is enabled
- **WHEN** a Planradar request fails transiently and is retried
- **THEN** each attempt is recorded as its own request event

#### Scenario: Uncaught frontend error
- **GIVEN** telemetry is enabled
- **WHEN** a rendering error or an unhandled promise rejection occurs in the user interface
- **THEN** an error event is recorded carrying the error name, a sanitized message, and the component or handler context

#### Scenario: Error is still shown to the user
- **WHEN** an error event is recorded
- **THEN** the German user-facing error message is displayed unchanged
- **AND** the displayed message does not depend on the outcome of telemetry recording
