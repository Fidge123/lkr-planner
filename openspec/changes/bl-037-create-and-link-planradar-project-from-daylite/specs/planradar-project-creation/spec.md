## ADDED Requirements

### Requirement: Create a blank Planradar project from a form
The system SHALL create a new Planradar project from a user-editable form, for an unlinked Daylite project.

#### Scenario: Submit create form
- **GIVEN** user has filled the create form
- **WHEN** user submits the form
- **THEN** a new project is created from the form values
- **AND** the new project ID is returned

#### Scenario: Default form values
- **GIVEN** user opens the create form without selecting a source project
- **WHEN** the form is shown
- **THEN** the project name defaults to the Daylite project name
- **AND** the remaining fields start empty

### Requirement: Create by copying a source project then editing
The system SHALL let the user create a project by copying an existing Planradar project, then editing the copy.

#### Scenario: Choose aspects to copy
- **GIVEN** user selects a source project
- **WHEN** configuring the copy
- **THEN** the user chooses a name and which aspects to copy (details, groups, ticket types, users, components)

#### Scenario: Copy then edit
- **GIVEN** user confirms the copy
- **WHEN** the project is created
- **THEN** the source project is copied via the copy-project endpoint with the selected toggles and name
- **AND** an edit form is opened to adjust the copied project's details before finishing
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