## MODIFIED Requirements

### Requirement: Read Request Optimization
The system SHALL serve repeated reads from caches, SHALL coalesce concurrent reads of the same record into a single request, and SHALL reuse open connections across commands.
Daylite cannot resolve a set of references in one search, so a week costs one request per distinct project reference that is not fresh in the cache.

#### Scenario: Repeated project resolution within the cache lifetime
- **GIVEN** a project reference resolved within the last five minutes
- **WHEN** the same reference is resolved again
- **THEN** the cached project is returned without a further API request

#### Scenario: Stale project served until its refresh succeeds
- **GIVEN** a project reference whose cache entry is older than five minutes
- **WHEN** the reference is resolved again and the API request fails
- **THEN** the stale cached project is returned

#### Scenario: Resolved projects survive a restart
- **GIVEN** projects resolved in an earlier session
- **WHEN** the application starts
- **THEN** those projects are read back from the app cache directory as stale entries
- **AND** a week loaded before any refresh shows their names and categories

#### Scenario: Concurrent resolution of one project reference
- **GIVEN** the same project reference is resolved concurrently, as a week and its prefetched neighbors do
- **WHEN** no fresh cache entry exists
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

#### Scenario: Connections are reused across commands
- **GIVEN** a Daylite command has completed a request
- **WHEN** another Daylite command sends a request to the same host
- **THEN** it reuses the open connection instead of opening a new one

#### Scenario: Contact list falls back to stale data
- **GIVEN** a contact list read encounters a transient failure
- **WHEN** contacts from an earlier read are still held in the cache
- **THEN** those contacts are served so the planning grid stays populated
- **AND** the German error message is surfaced alongside them
