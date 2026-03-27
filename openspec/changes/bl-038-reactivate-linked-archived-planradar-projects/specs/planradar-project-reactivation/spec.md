## ADDED Requirements

### Requirement: Detect archived projects
The system SHALL detect linked Planradar projects that are archived.

#### Scenario: Fetch project status
- **GIVEN** a linked Planradar project ID
- **WHEN** fetching project status
- **THEN** the current status (active/archived/closed) is returned

#### Scenario: Identify archived status
- **GIVEN** a project with archived status
- **WHEN** checking status
- **THEN** the project is identified as archived

### Requirement: Reactivate archived projects
The system SHALL reactivate archived linked projects.

#### Scenario: Reactivate archived project
- **GIVEN** an archived linked project
- **WHEN** reactivating the project
- **THEN** the project status changes to active
- **AND** success is logged

#### Scenario: Skip already active project
- **GIVEN** an active linked project
- **WHEN** reactivating the project
- **THEN** no API call is made
- **AND** success is returned (idempotent)

#### Scenario: Handle missing project
- **GIVEN** a linked project ID that doesn't exist in Planradar
- **WHEN** attempting reactivation
- **THEN** an error is returned
- **AND** sync issue is logged

### Requirement: Logging
The system SHALL log reactivation actions.

#### Scenario: Log successful reactivation
- **GIVEN** project is successfully reactivated
- **WHEN** logging the action
- **THEN** log includes project ID, name, and timestamp

#### Scenario: Log failed reactivation
- **GIVEN** reactivation fails
- **WHEN** logging the failure
- **THEN** log includes project ID, error message, and timestamp