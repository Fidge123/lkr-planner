## Requirements

### Requirement: API Authentication & Token Rotation
The system SHALL authenticate with Daylite API and persist refresh-token rotation states.

#### Scenario: Handle Token Expiration
- **GIVEN** an expired Daylite access token and a valid refresh token
- **WHEN** making a Daylite API request
- **THEN** the system seamlessly rotates the tokens
- **AND** saves the new token state
- **AND** completes the original request

### Requirement: Project and Contact Search
The system SHALL support typed read and search commands for Daylite Projects and Contacts.

#### Scenario: Retrieve Projects
- **GIVEN** an active Daylite session
- **WHEN** the application requests the project list
- **THEN** Daylite projects are retrieved, parsed into domain models, and returned

### Requirement: Read Request Optimization
The system SHALL optimize repeated reads using a short-lived in-memory TTL cache and request coalescing.

#### Scenario: Coalescing simultaneous requests
- **GIVEN** multiple UI components request the same Daylite project data concurrently
- **WHEN** no valid cache exists
- **THEN** only one underlying API request is dispatched
- **AND** the result is distributed to all pending callers

#### Scenario: Graceful Stale Fallback
- **GIVEN** a request to Daylite encounters transient network failure
- **WHEN** stale data is present in the cache
- **THEN** the system falls back to the stale data to maintain UI stability

### Requirement: Project search with server-side status filtering
The system SHALL filter projects by status in the Daylite API search body.

#### Scenario: Search projects with text query
- **WHEN** user searches for projects with a query string
- **AND** query is at least 1 character
- **THEN** the service returns projects matching the query by name
- **AND** only projects with status `new_status` or `in_progress` are included
- **AND** status filter is applied server-side using a single request with array body (OR logic)
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
