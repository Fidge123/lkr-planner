## MODIFIED Requirements

### Requirement: Two-tier event display
The system SHALL distinguish lkr-planner assignments from bare calendar events.

#### Scenario: Display lkr-planner assignment
- **WHEN** a VEVENT has a DESCRIPTION first line matching `daylite:/<path>`
- **THEN** it is shown with the color of its Daylite project's category (`hex_colour`)
- **AND** an edit affordance is shown

#### Scenario: Assignment without a category color
- **WHEN** a resolved Daylite project has no category or its category has no color
- **THEN** the event is shown with the color derived from the Daylite project status instead

#### Scenario: Readable text on category color
- **WHEN** an assignment event is rendered with a category color
- **THEN** the text color is chosen by the relative luminance of the category color so the title stays readable

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
- **AND** the status color is used when no category color is cached

#### Scenario: Project not in cache — API fallback
- **WHEN** a VEVENT references a Daylite project
- **AND** the project is not in the local cache
- **THEN** the system queries the Daylite API for the project details including its category
- **AND** displays the resolved name and category color on success
- **AND** the status color is used when the project has no category color

#### Scenario: Project resolution fails
- **WHEN** a VEVENT references a Daylite project
- **AND** neither cache lookup nor API query succeeds
- **THEN** a German placeholder is shown: `"Beschreibung für [event SUMMARY] konnte nicht abgerufen werden"`
- **AND** neutral color is used
