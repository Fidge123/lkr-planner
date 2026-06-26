## ADDED Requirements

### Requirement: Move assignment between calendars
The system SHALL move an assignment from one employee's CalDAV calendar to another employee's CalDAV calendar in a single operation and report whether the move completed fully.

#### Scenario: Move to another employee's calendar
- **WHEN** a move is requested with the source assignment href and a target employee reference and date
- **THEN** a new VEVENT carrying the same project reference, project name, and time-of-day is created on the target employee's primary calendar at the target date
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
