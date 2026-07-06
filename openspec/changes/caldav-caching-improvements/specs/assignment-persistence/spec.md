## MODIFIED Requirements

### Requirement: Load assignments from CalDAV
The system SHALL load assignment events from each employee's primary CalDAV calendar, served through the backend's CalDAV event cache.

#### Scenario: Load assignments for week
- **WHEN** user navigates to a week
- **THEN** VEVENTs for that week are fetched from the backend cache when a fresh entry exists, or from each employee's primary CalDAV calendar otherwise
- **AND** displayed in the planning grid

#### Scenario: No events exist for week
- **WHEN** loading events for a week with no calendar entries
- **THEN** empty cells are shown
- **AND** user can create new assignments (via BL-016)

#### Scenario: Employee has no primary calendar configured
- **WHEN** an employee has no `zepPrimaryCalendar` setting
- **THEN** their row shows empty cells without triggering a fetch or error

### Requirement: Week navigation with live data
The system SHALL use CalDAV, via the backend cache, as the data source for all week navigation, keeping data fresh through targeted invalidation rather than a full cache wipe.

#### Scenario: Navigate between weeks
- **WHEN** user navigates to a different week
- **THEN** cached or freshly fetched CalDAV data for the new week's date range is displayed

#### Scenario: Pre-fetch adjacent weeks
- **WHEN** a week is loaded
- **THEN** the previous and next weeks are silently pre-fetched into the frontend cache
- **AND** navigation to an adjacent week displays instantly without a loading state

#### Scenario: Cache invalidated after own write
- **WHEN** the user creates, updates, or deletes an assignment
- **THEN** only the affected employee's week cache entry is invalidated in both the backend and frontend caches
- **AND** other cached weeks remain untouched
- **AND** the affected week reflects the change on next display without a full application reload
