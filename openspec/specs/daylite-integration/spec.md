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
