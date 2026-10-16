## Purpose

Provide the backend's Daylite API client: authentication with refresh-token rotation, typed project and contact reads, cached and coalesced requests, and normalized German error messages.

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

#### Scenario: Retrieve Contacts
- **GIVEN** an active Daylite session
- **WHEN** the application requests the contact list
- **THEN** Daylite contacts are retrieved, parsed into domain models, and returned

#### Scenario: Resolve a single project by reference
- **GIVEN** an assignment carrying a Daylite project reference
- **WHEN** the project behind that reference is requested
- **THEN** the project is retrieved and its name, status, and category are returned
- **AND** an unrecognized status is normalized to `new_status`

### Requirement: Read Request Optimization
The system SHALL serve repeated reads from short-lived in-memory caches, and SHALL coalesce concurrent reads of the same record into a single request.
Daylite cannot resolve a set of references in one search, so a week costs one request per distinct project reference it has not cached.

#### Scenario: Repeated project resolution within the cache lifetime
- **GIVEN** a project reference resolved while loading a week
- **WHEN** the same reference is resolved again before the cache entry expires
- **THEN** the cached project is returned without a further API request

#### Scenario: Concurrent resolution of one project reference
- **GIVEN** the same project reference is resolved concurrently, as a week and its prefetched neighbors do
- **WHEN** no valid cache entry exists
- **THEN** one API request is dispatched
- **AND** every caller receives its result

#### Scenario: Failed project resolution is not cached
- **GIVEN** a project reference whose resolution failed
- **WHEN** the same reference is resolved again
- **THEN** the resolution is retried instead of serving the failure

#### Scenario: Project resolution stays within its concurrency limit
- **GIVEN** a week referencing more uncached projects than the concurrency limit
- **WHEN** those references are resolved
- **THEN** no more than that many requests are in flight at once

#### Scenario: Contact list falls back to stale data
- **GIVEN** a contact list read encounters a transient failure
- **WHEN** contacts from an earlier read are still held in the cache
- **THEN** those contacts are served so the planning grid stays populated
- **AND** the German error message is surfaced alongside them

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

### Requirement: Category color retrieval
The system SHALL retrieve Daylite categories with their colors for coloring project events.

#### Scenario: Fetch project categories
- **WHEN** the category colors are requested
- **THEN** the system requests `GET /categories` with the `entity=project` filter
- **AND** parses `name` and `hex_colour` for each category into a name-to-color map

#### Scenario: Category without a color
- **WHEN** a category's `hex_colour` is null
- **THEN** the category yields no color
- **AND** events of projects in that category fall back to the neutral color

#### Scenario: One map serves the grid and the assignment modal
- **WHEN** the frontend requests the project category colors
- **THEN** the name-to-color map is returned as a typed command result
- **AND** the planning grid and the project picker coloring both read that one map
- **AND** a failure yields no colors rather than a user-facing error

#### Scenario: Overdue results carry their category
- **WHEN** overdue projects are queried
- **THEN** each result carries the category `"Überfällig"` the query filtered on, even though Daylite omits the field from these minimal records

#### Scenario: Inactive categories keep their color
- **WHEN** a category has `is_active` set to false
- **AND** an existing project still references that category
- **THEN** the category's color is still used for that project's events
