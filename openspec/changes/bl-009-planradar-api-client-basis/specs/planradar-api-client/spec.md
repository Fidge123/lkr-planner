## ADDED Requirements

### Requirement: Authentication
The system SHALL authenticate Planradar requests with a static, user-provided API token and target the configured customer.

#### Scenario: Authenticated request
- **WHEN** the client sends a request to the Planradar API
- **THEN** the configured API token is attached in the `X-PlanRadar-API-Key` header
- **AND** the request targets the configured Customer ID path segment
- **AND** no OAuth flow or token rotation is performed

#### Scenario: Missing or invalid token
- **WHEN** no token is configured or Planradar rejects the token
- **THEN** a normalized authentication error is returned
- **AND** the user is prompted to provide a valid token

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
The system SHALL provide functionality to create new Planradar projects, either blank or copied from a source project.

Blank creation uses the create-project endpoint with project attributes (name, address, dates, description).
Copying uses the dedicated copy-project endpoint with a new name and per-aspect toggles for details, groups, ticket types (forms), users, and components (layers).

#### Scenario: Create a blank project
- **WHEN** user creates a project without a source project
- **THEN** a new project is created from the provided attributes
- **AND** the new project ID is returned

#### Scenario: Copy from a source project
- **WHEN** user creates a project from a source project
- **THEN** the source project is copied via the copy-project endpoint with the new name and selected aspect toggles
- **AND** the new project ID is returned

### Requirement: Project status read
The system SHALL provide functionality to read project status.

#### Scenario: Read project status
- **WHEN** user requests status for a specific project
- **THEN** the client returns project status (active/archived)
- **AND** can determine if project can be reopened

#### Scenario: Reactivate archived project
- **WHEN** user requests to reopen an archived project
- **THEN** the archive-project endpoint is called with status set to active (status `1`)
- **AND** the project status changes to active
- **AND** success confirmation is returned

### Requirement: Error normalization
The system SHALL normalize API errors into consistent format.

#### Scenario: Handle API errors
- **WHEN** Planradar API returns an error
- **THEN** error is converted to standardized internal error type
- **AND** error includes status code and message for debugging