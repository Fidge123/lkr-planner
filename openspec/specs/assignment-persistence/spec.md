## Purpose

Make CalDAV the live source of truth for the planning grid by loading each employee's calendar events per week, resolving the Daylite project behind each assignment, and moving assignments between employee calendars.

## Requirements

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
- **THEN** it is shown on a neutral card carrying a strip in its Daylite project's category color (`hex_colour`)
- **AND** an edit affordance is shown

#### Scenario: Category color is applied verbatim
- **WHEN** an assignment event is rendered with a category color
- **THEN** the value from Daylite is used as-is, so any CSS color notation it uses still applies
- **AND** it colors only the strip, leaving the card surface and its text unchanged

#### Scenario: Assignment without a category color
- **WHEN** a resolved Daylite project has no category or its category has no color
- **THEN** the strip keeps its default muted color, the same one for every project status

#### Scenario: Bare events carry no strip
- **WHEN** an event has no Daylite project reference
- **THEN** it is rendered without a strip
- **AND** it shares the same surface color as an assignment, so the strip alone marks an event as an assignment

#### Scenario: Display bare event
- **WHEN** a VEVENT has no structured Daylite project reference
- **THEN** it is shown with neutral/grey styling
- **AND** no edit affordance is shown (read-only)
- **AND** covers legacy manually-created events and employee blockers

#### Scenario: Display event start and end time
- **WHEN** a VEVENT has a start time (non-all-day event)
- **THEN** the start time is shown in HH:MM format on the left of the event card
- **AND** the end time is shown below the start time if present
- **AND** all-day events show no time

#### Scenario: Event card hover feedback
- **WHEN** the user hovers over any event card
- **THEN** a visual hover indicator is shown

#### Scenario: Long event titles
- **WHEN** an event title exceeds the card width
- **THEN** the title wraps to multiple lines
- **AND** the card grows vertically to accommodate the text

### Requirement: Daylite project resolution
The system SHALL resolve project details for lkr-planner events.

#### Scenario: Project found in cache
- **WHEN** a VEVENT references a Daylite project
- **AND** the project is present in the local Daylite cache
- **THEN** the project name and category color are displayed from cache
- **AND** the neutral color is used when no category color is cached

#### Scenario: Project not in cache — API fallback
- **WHEN** a VEVENT references a Daylite project
- **AND** the project is not in the local cache
- **THEN** the system queries the Daylite API for the project details including its category
- **AND** displays the resolved name and category color on success
- **AND** the neutral color is used when the project has no category color

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

#### Scenario: Pre-fetch adjacent weeks
- **WHEN** a week is loaded
- **THEN** the previous and next weeks are silently pre-fetched into the cache
- **AND** navigation to an adjacent week displays instantly without a loading state

### Requirement: Loading and error states
The system SHALL maintain German loading and error states.

#### Scenario: Show loading state
- **WHEN** CalDAV events are being fetched
- **THEN** a German loading indicator is shown above the planning table

#### Scenario: Show error state on per-employee fetch failure
- **WHEN** a CalDAV fetch fails for an individual employee
- **THEN** their row shows a German error indicator: "Kalender nicht verfügbar"
- **AND** a retry button is shown that re-fetches the week for all employees

#### Scenario: Show error state on total fetch failure
- **WHEN** the calendar data cannot be loaded at all (store unavailable, bad date)
- **THEN** a German error banner is displayed above the planning table
- **AND** user can retry the fetch

### Requirement: Move assignment between calendars
The system SHALL move an assignment from one employee's CalDAV calendar to another employee's CalDAV calendar in a single operation and report whether the move completed fully.

#### Scenario: Move to another employee's calendar
- **WHEN** a move is requested with the source assignment href and a target employee reference and date
- **THEN** a new VEVENT carrying the same project reference and project name is created on the target employee's primary calendar at the target date with the standard assignment time window
- **AND** the original VEVENT is deleted from the source calendar
- **AND** a result indicating a full move with the new CalDAV href is returned

#### Scenario: Source delete fails after target create
- **WHEN** the target VEVENT is created but deleting the source VEVENT fails
- **THEN** the source VEVENT is left in place
- **AND** a result is returned indicating a partial move, carrying both the new href and the source href
- **AND** the operation does not report a plain success

#### Scenario: Target employee has no primary calendar
- **WHEN** a move targets an employee without a configured primary calendar
- **THEN** the operation fails with a German error message
- **AND** the source assignment is left untouched

#### Scenario: Refuse moves into an absence calendar
- **WHEN** a move would write into a configured absence calendar
- **THEN** the operation is refused with a German error message
- **AND** the source assignment is left untouched

#### Scenario: Target create fails
- **WHEN** creating the VEVENT on the target calendar fails
- **THEN** the source VEVENT is not deleted
- **AND** a German error message is returned
