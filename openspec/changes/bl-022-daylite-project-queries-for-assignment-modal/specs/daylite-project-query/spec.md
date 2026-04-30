## ADDED Requirements

### Requirement: Project search with server-side status filtering
The system SHALL filter projects by status in the Daylite API search body.

#### Scenario: Search projects with text query
- **WHEN** user searches for projects with a query string
- **AND** query is at least 1 character
- **THEN** the service returns projects matching the query by name
- **AND** only projects with status `new_status` or `in_progress` are included
- **AND** status filter is applied server-side in the Daylite search body
- **AND** results are limited to 5 items

#### Scenario: Search returns deterministic results
- **WHEN** user performs the same search twice
- **THEN** the results are identical
- **AND** projects are sorted by numeric project ID ascending

#### Scenario: Backwards-compatible search without status filter
- **WHEN** caller provides no status filter
- **THEN** search body contains no status constraint
- **AND** all statuses are returned as before

### Requirement: Timeout error handling
The system SHALL return a German error message when the Daylite API request times out.

#### Scenario: Handle API timeout
- **WHEN** Daylite API request times out (after 5 seconds)
- **THEN** return error code `Timeout`
- **AND** return user message `"Zeitüberschreitung bei der Daylite-Anfrage"`

### Requirement: Error normalization
The system SHALL normalize malformed API responses into German user-facing error messages.

#### Scenario: Handle malformed response
- **WHEN** Daylite API returns unexpected response format
- **THEN** return error code `InvalidResponse`
- **AND** return user message `"Ungültige Antwort von Daylite"`
