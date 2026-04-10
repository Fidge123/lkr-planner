## ADDED Requirements

### Requirement: Load assignments from CalDAV
The system SHALL load assignment events from each employee's primary CalDAV calendar.

#### Scenario: Load assignments for week
- **WHEN** user navigates to a week
- **THEN** VEVENTs for that week are fetched from each employee's primary CalDAV calendar
- **AND** displayed in the planning grid

#### Scenario: No events exist for week
- **WHEN** loading events for a week with no calendar entries
- **THEN** empty cells are shown
- **AND** user can create new assignments (via BL-016)

#### Scenario: Employee has no primary calendar configured
- **WHEN** an employee has no `zepPrimaryCalendar` setting
- **THEN** their row shows empty cells without triggering a fetch or error

### Requirement: Two-tier event display
The system SHALL distinguish lkr-planner assignments from bare calendar events.

#### Scenario: Display lkr-planner assignment
- **WHEN** a VEVENT has a DESCRIPTION first line matching `daylite:/<path>`
- **THEN** it is shown with project color derived from Daylite status
- **AND** an edit affordance is shown

#### Scenario: Display bare event
- **WHEN** a VEVENT has no structured Daylite project reference
- **THEN** it is shown with neutral/grey styling
- **AND** no edit affordance is shown (read-only)
- **AND** covers legacy manually-created events and employee blockers

### Requirement: Daylite project resolution
The system SHALL resolve project details for lkr-planner events.

#### Scenario: Project found in cache
- **WHEN** a VEVENT references a Daylite project
- **AND** the project is present in the local Daylite cache
- **THEN** the project name and status color are displayed from cache

#### Scenario: Project not in cache — API fallback
- **WHEN** a VEVENT references a Daylite project
- **AND** the project is not in the local cache
- **THEN** the system queries the Daylite API for the project details
- **AND** displays the resolved name and status color on success

#### Scenario: Project resolution fails
- **WHEN** a VEVENT references a Daylite project
- **AND** neither cache lookup nor API query succeeds
- **THEN** a German placeholder is shown: `"Beschreibung für [event SUMMARY] konnte nicht abgerufen werden"`
- **AND** neutral color is used

### Requirement: Week navigation with live data
The system SHALL use CalDAV as the data source for all week navigation.

#### Scenario: Navigate between weeks
- **WHEN** user navigates to a different week
- **THEN** CalDAV is queried for the new week's date range
- **AND** assignments for the new week are displayed

### Requirement: Loading and error states
The system SHALL maintain German loading and error states.

#### Scenario: Show loading state
- **WHEN** CalDAV events are being fetched
- **THEN** a German loading indicator is shown

#### Scenario: Show error state on fetch failure
- **WHEN** a CalDAV fetch fails for one or more employees
- **THEN** a German error message is displayed
- **AND** user can retry the fetch
