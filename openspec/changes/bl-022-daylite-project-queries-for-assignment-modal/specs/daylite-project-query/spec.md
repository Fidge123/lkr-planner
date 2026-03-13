## ADDED Requirements

### Requirement: Project search with status filtering
The system SHALL provide project search that returns only new and in-progress projects.

#### Scenario: Search projects with text query
- **WHEN** user searches for projects with a query string
- **AND** query is at least 1 character
- **THEN** the service returns projects matching the query
- **AND** only projects with status `new_status` or `in_progress` are included
- **AND** results are limited to 5 items

#### Scenario: Search returns deterministic results
- **WHEN** user performs the same search twice
- **THEN** the results are identical
- **AND** projects are sorted by ID ascending

### Requirement: Overdue project query
The system SHALL provide a query for overdue projects used by default suggestions.

#### Scenario: Query overdue projects
- **WHEN** user requests overdue projects
- **THEN** the service returns projects with status indicating overdue
- **AND** results are limited to 5 items
- **AND** results are sorted by ID ascending

### Requirement: Error normalization
The system SHALL normalize API errors into German user-facing error messages.

#### Scenario: Handle API timeout
- **WHEN** Daylite API request times out
- **THEN** return error message "Zeitüberschreitung bei der Daylite-Anfrage"

#### Scenario: Handle malformed response
- **WHEN** Daylite API returns unexpected response format
- **THEN** return error message "Ungültige Antwort von Daylite"
