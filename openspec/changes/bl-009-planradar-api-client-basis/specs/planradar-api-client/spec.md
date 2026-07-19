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

### Requirement: Project list
The system SHALL provide functionality to list Planradar projects with pagination.

The Planradar list endpoint (`GET .../projects`) supports `sort`, `page`, and `pagesize` query parameters but no full-text query, so listing is paginated rather than searched.

#### Scenario: List a page of projects
- **WHEN** user requests projects with optional sort, page, and pagesize parameters
- **THEN** the client returns the requested page of accessible projects
- **AND** results are typed as Project structs

#### Scenario: Page through all projects
- **WHEN** user needs all accessible projects
- **THEN** the caller requests successive pages via the page and pagesize parameters
- **AND** the client returns one page per call (pagination is caller-driven, not aggregated)

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