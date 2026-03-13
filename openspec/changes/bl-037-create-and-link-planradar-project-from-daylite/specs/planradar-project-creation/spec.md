## ADDED Requirements

### Requirement: Create Planradar project
The system SHALL create a new Planradar project from an unlinked Daylite project.

#### Scenario: Create from template
- **GIVEN** user selects template project
- **WHEN** creating Planradar project
- **THEN** a new project is created with template's properties
- **AND** the new project ID is returned

#### Scenario: Create without template
- **GIVEN** user does not select template
- **WHEN** creating Planradar project
- **THEN** a new project is created with Daylite project name
- **AND** the new project ID is returned

### Requirement: Idempotent creation
The system SHALL prevent duplicate Planradar project creation.

#### Scenario: Existing link found
- **GIVEN** Daylite project has existing Planradar ID in custom field
- **AND** the Planradar project exists
- **WHEN** user initiates create
- **THEN** existing project is returned (no new creation)

#### Scenario: Stale link (orphan Planradar)
- **GIVEN** Daylite project has Planradar ID but project missing in Planradar
- **WHEN** user initiates create
- **THEN** a sync issue is logged
- **AND** new project is created

### Requirement: Persist link
The system SHALL persist created Planradar ID to Daylite.

#### Scenario: Write link to Daylite
- **GIVEN** new Planradar project is created
- **WHEN** persisting the link
- **THEN** Planradar ID is written to Daylite custom field
- **AND** operation succeeds

#### Scenario: Write failure handling
- **GIVEN** write to Daylite fails
- **WHEN** persisting the link
- **THEN** operation is queued for retry
- **AND** sync issue is logged

### Requirement: Template selection
The system SHALL allow user to filter and select template projects.

#### Scenario: Filter templates
- **GIVEN** user types search filter
- **WHEN** filtering template list
- **THEN** only matching projects are shown

#### Scenario: Select template
- **GIVEN** user selects a template project
- **WHEN** confirming selection
- **THEN** template ID is stored for creation