## ADDED Requirements

### Requirement: Backend caches CalDAV week events
The system SHALL cache resolved CalDAV week events in the backend, keyed by employee, calendar URL, and week start.

#### Scenario: Cache hit within TTL
- **WHEN** `load_week_events` is called for an employee/week already cached within the last 30 seconds
- **THEN** the cached event list is returned
- **AND** no CalDAV request is issued

#### Scenario: Cache miss after TTL expiry
- **WHEN** `load_week_events` is called for an employee/week whose cache entry is older than 30 seconds
- **THEN** a fresh CalDAV request is issued
- **AND** the cache entry is replaced with the new result and timestamp

#### Scenario: Cache miss for uncached week
- **WHEN** `load_week_events` is called for an employee/week with no existing cache entry
- **THEN** a CalDAV request is issued
- **AND** the result is stored in the cache

### Requirement: Concurrent identical fetches are coalesced
The system SHALL coalesce concurrent identical CalDAV week-event fetches into a single in-flight request.

#### Scenario: Rapid duplicate requests
- **WHEN** two calls for the same employee/calendar/week arrive while a fetch for that key is already in flight
- **THEN** both calls await the same underlying CalDAV request
- **AND** exactly one CalDAV request is issued

### Requirement: Targeted cache invalidation on write
The system SHALL invalidate only the affected employee/week cache entry when an assignment is created, updated, or deleted.

#### Scenario: Invalidate on create
- **WHEN** `create_assignment` succeeds
- **THEN** the cache entry for that employee's calendar and the assignment's week is removed
- **AND** other employees' and weeks' cache entries remain unaffected

#### Scenario: Invalidate on update
- **WHEN** `update_assignment` succeeds
- **THEN** the cache entry for that employee's calendar and the assignment's week is removed

#### Scenario: Invalidate on delete
- **WHEN** `delete_assignment` succeeds
- **THEN** the cache entry for that employee's calendar and the assignment's week is removed

### Requirement: Stale-on-error fallback
The system SHALL serve the last cached result if a refresh fetch fails and a previous successful entry exists.

#### Scenario: Refresh fails with existing cache entry
- **WHEN** a CalDAV fetch fails
- **AND** a previous successful cache entry exists for that employee/calendar/week
- **THEN** the stale cached data is returned
- **AND** no error is surfaced to the caller for that fetch

#### Scenario: Refresh fails with no existing cache entry
- **WHEN** a CalDAV fetch fails
- **AND** no previous cache entry exists for that employee/calendar/week
- **THEN** the failure is propagated as an error

### Requirement: Shared HTTP client for CalDAV requests
The system SHALL reuse a single HTTP client instance across all CalDAV commands instead of constructing a new client per call.

#### Scenario: Multiple CalDAV commands reuse the same client
- **WHEN** `load_week_events`, `create_assignment`, `update_assignment`, and `delete_assignment` are called during a session
- **THEN** all of them use the same shared HTTP client instance
- **AND** no per-call client construction occurs
