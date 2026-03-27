## ADDED Requirements

### Requirement: Project search and list
The system SHALL provide functionality to search and list Planradar projects.

#### Scenario: Search projects by query
- **WHEN** user searches for projects with a query string
- **THEN** the client returns projects matching the query
- **AND** results are typed as Project structs

#### Scenario: List all accessible projects
- **WHEN** user requests all projects
- **THEN** the client returns a list of all accessible projects
- **AND** pagination is handled automatically

### Requirement: Project create
The system SHALL provide functionality to create new Planradar projects.

#### Scenario: Create project from template
- **WHEN** user creates a project using an existing project as template
- **THEN** a new project is created in Planradar
- **AND** the new project ID is returned

### Requirement: Project status read
The system SHALL provide functionality to read project status.

#### Scenario: Read project status
- **WHEN** user requests status for a specific project
- **THEN** the client returns project status (active/archived)
- **AND** can determine if project can be reopened

#### Scenario: Reactivate archived project
- **WHEN** user requests to reopen an archived project
- **THEN** the project status changes to active
- **AND** success confirmation is returned

### Requirement: Error normalization
The system SHALL normalize API errors into consistent format.

#### Scenario: Handle API errors
- **WHEN** Planradar API returns an error
- **THEN** error is converted to standardized internal error type
- **AND** error includes status code and message for debugging