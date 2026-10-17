# Spec Delta

## MODIFIED Requirements

### Requirement: Two-tier event display
The system SHALL distinguish lkr-planner assignments and custom appointments from bare calendar events.

#### Scenario: Display lkr-planner assignment
- **WHEN** a VEVENT has a DESCRIPTION first line matching `daylite:/<path>`
- **THEN** it is shown on a neutral card carrying a strip in its Daylite project's category color (`hex_colour`)
- **AND** an edit affordance is shown

#### Scenario: Category color is applied verbatim
- **WHEN** an assignment event is rendered with a category color
- **THEN** the value from Daylite is used as-is, so any CSS color notation it uses still applies
- **AND** it colors only the strip, leaving the card surface and its text unchanged

#### Scenario: Assignment without a category color
- **WHEN** a resolved Daylite project has no category, or its category has no color
- **THEN** the strip keeps its default muted color, the same one for every project status

#### Scenario: Category color is joined where the card is rendered
- **WHEN** an assignment card is rendered
- **THEN** its strip color comes from looking the project's category up in the category color map
- **AND** the map is read once for the whole session, so a recolored category appears after a reload or a restart

#### Scenario: Bare events carry no strip
- **WHEN** an event is a bare event
- **THEN** it is rendered without a strip
- **AND** it shares the same surface color as an assignment, so the strip alone marks an event as an assignment or a custom appointment

#### Scenario: Display custom appointment
- **WHEN** a VEVENT carries the custom appointment marker and no Daylite project reference
- **THEN** it is shown on a neutral card carrying a strip in the default muted color
- **AND** an edit affordance is shown

#### Scenario: Display bare event
- **WHEN** a VEVENT has no structured Daylite project reference
- **AND** it carries no custom appointment marker
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

### Requirement: Move assignment between calendars
The system SHALL move an assignment or a custom appointment from one employee's CalDAV calendar to another employee's CalDAV calendar in a single operation and report whether the move completed fully.

#### Scenario: Move to another employee's calendar
- **WHEN** a move is requested with the source assignment href and a target employee reference and date
- **THEN** a new VEVENT carrying the same project reference and project name, or the same custom title and custom appointment marker, is created on the target employee's primary calendar at the target date with the standard assignment time window
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
