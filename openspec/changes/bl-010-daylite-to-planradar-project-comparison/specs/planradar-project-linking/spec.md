## ADDED Requirements

### Requirement: Check existing link
The system SHALL determine if a Daylite project has a linked Planradar project.

#### Scenario: Project has existing link
- **WHEN** checking a Daylite project that has a Planradar link
- **THEN** the Planradar project ID is returned
- **AND** the link can be used for further operations

#### Scenario: Project has no link
- **WHEN** checking a Daylite project without a Planradar link
- **THEN** null/no link is returned
- **AND** user can initiate a new link operation

### Requirement: Link to existing Planradar project
The system SHALL allow linking a Daylite project to an existing Planradar project.

#### Scenario: Create new link
- **WHEN** user selects a Planradar project to link
- **THEN** the Planradar project ID is persisted to Daylite custom field
- **AND** the link is available for subsequent operations

### Requirement: Idempotent linking
The system SHALL ensure link operations are idempotent.

#### Scenario: Repeated link operation
- **WHEN** linking a Daylite project that already has a link
- **THEN** the existing link is reused
- **AND** no duplicate writes occur

### Requirement: Link operation logging
The system SHALL log link operations as sync events.

#### Scenario: Log link creation
- **WHEN** a new link is created
- **THEN** the operation is logged with timestamp and details
- **AND** the log entry is accessible for troubleshooting