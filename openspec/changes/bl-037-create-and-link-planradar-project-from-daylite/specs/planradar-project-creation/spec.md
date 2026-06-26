## ADDED Requirements

### Requirement: Create Planradar project from a form
The system SHALL create a new Planradar project from a user-editable form, for an unlinked Daylite project.

#### Scenario: Submit create form
- **GIVEN** user has filled the create form
- **WHEN** user submits the form
- **THEN** a new project is created from the form values
- **AND** the new project ID is returned

#### Scenario: Default form without pre-fill
- **GIVEN** user opens the create form without selecting a source project
- **WHEN** the form is shown
- **THEN** the project name defaults to the Daylite project name
- **AND** the remaining fields start empty

### Requirement: Pre-fill the create form from a source project
The system SHALL let the user pre-fill the create form from an existing Planradar project.

#### Scenario: Pre-fill plausible fields
- **GIVEN** user selects a source project
- **WHEN** the form is pre-filled
- **THEN** the source project's data is read
- **AND** fields where reuse is plausible are copied from the source project
- **AND** fields where reuse is not plausible are left empty

#### Scenario: Edit pre-filled fields before submit
- **GIVEN** the form has been pre-filled from a source project
- **WHEN** user edits any field
- **THEN** the edited values are used on submit instead of the source values

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

### Requirement: Source project selection
The system SHALL allow user to filter and select a source project.

#### Scenario: Filter source projects
- **GIVEN** user types search filter
- **WHEN** filtering the source project list
- **THEN** only matching projects are shown

#### Scenario: Select source project
- **GIVEN** user selects a source project
- **WHEN** confirming selection
- **THEN** the source project ID is stored for creation