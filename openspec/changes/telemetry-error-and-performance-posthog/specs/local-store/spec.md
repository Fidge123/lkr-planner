## ADDED Requirements

### Requirement: Persisted Telemetry Preference
The system SHALL persist the telemetry opt-in flag and the anonymous install identifier in the local store, and SHALL default the flag to disabled when it is absent.

#### Scenario: Telemetry preference persisted
- **GIVEN** the user changes the telemetry setting
- **WHEN** the local store is saved and reloaded
- **THEN** the restored configuration reports the same telemetry state

#### Scenario: Store written before telemetry existed
- **GIVEN** a stored configuration file without a telemetry section
- **WHEN** the local store is loaded
- **THEN** loading succeeds without error
- **AND** telemetry is reported as disabled

#### Scenario: Install identifier persisted
- **GIVEN** an install identifier has been generated
- **WHEN** the local store is saved and reloaded
- **THEN** the restored configuration reports the same identifier
