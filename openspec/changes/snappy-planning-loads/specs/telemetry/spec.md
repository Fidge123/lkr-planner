## MODIFIED Requirements

### Requirement: Performance events
The system SHALL record the measured duration of backend commands, outbound integration requests, and application startup, and SHALL record background work under operation names distinct from the waits a user sees.

#### Scenario: Backend command duration
- **GIVEN** telemetry is enabled
- **WHEN** a backend command completes, successfully or with an error
- **THEN** a performance event is recorded carrying the operation name, the duration in milliseconds, and whether the operation succeeded

#### Scenario: Outbound request duration
- **GIVEN** telemetry is enabled
- **WHEN** a request to an integration completes
- **THEN** a performance event is recorded carrying the integration name, the request kind, the duration in milliseconds, and the response status when one was received

#### Scenario: Background week loads are distinguishable
- **GIVEN** telemetry is enabled
- **WHEN** a week is loaded to prefetch it, or to refresh the rows of an edit
- **THEN** its performance event carries the operation `prefetch_week_events` or `refresh_week_events` instead of `load_week_events`

#### Scenario: Project resolution is recorded on its own
- **GIVEN** telemetry is enabled
- **WHEN** the Daylite projects of a week are resolved
- **THEN** a performance event with the operation `resolve_week_projects` is recorded
- **AND** each project lookup is recorded as an outbound Daylite request

#### Scenario: Startup duration
- **GIVEN** telemetry is enabled
- **WHEN** the application finishes starting up
- **THEN** a performance event is recorded carrying the duration from process start to the ready state
