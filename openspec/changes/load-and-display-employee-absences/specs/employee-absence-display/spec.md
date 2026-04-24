## ADDED Requirements

### Requirement: Employee absence fetching
The system SHALL fetch absence events from each employee's configured ZEP absence calendar when loading week events.

#### Scenario: Absence events included in week response
- **WHEN** `load_week_events` is called for a week
- **THEN** absence events from `zep_absence_calendar` are included in `EmployeeWeekEvents.events` for each employee
- **AND** each absence event has `kind: Absence`

#### Scenario: No absence calendar configured
- **WHEN** an employee has no `zep_absence_calendar` set
- **THEN** no absence events are returned for that employee
- **AND** the response is otherwise unaffected

#### Scenario: Absence calendar fetch failure
- **WHEN** fetching the absence calendar fails (network error, auth error, etc.)
- **THEN** no absence events are returned for that employee
- **AND** the primary calendar events and `error` field are unaffected

#### Scenario: Absence event carries summary as title
- **WHEN** an absence calendar event has a SUMMARY field
- **THEN** the absence event `title` is set to that SUMMARY value
- **AND** `project_status` is `null`

### Requirement: Absence event display
The system SHALL render absence events visually distinct from assignment and bare events in the timetable.

#### Scenario: Absence cell styling
- **WHEN** a timetable cell contains an absence event
- **THEN** the absence event is rendered non-interactively (no button)
- **AND** it uses a warning color distinct from assignment and bare event colors

#### Scenario: Multiple event types in same cell
- **WHEN** a cell contains both an absence event and an assignment event
- **THEN** both are rendered in the cell
- **AND** each uses its respective styling
