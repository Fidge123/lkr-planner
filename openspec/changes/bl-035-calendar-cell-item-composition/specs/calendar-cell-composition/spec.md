## ADDED Requirements

### Requirement: Normalized item model
The system SHALL provide a normalized `CalendarCellItem` model for all cell contents.

#### Scenario: Model fields
- **WHEN** a calendar cell item is created
- **THEN** it contains: id, type, title, startTime, isReadOnly, sourceData

### Requirement: Compose from all sources
The system SHALL compose calendar cell items from all required sources.

#### Scenario: Absence items included
- **GIVEN** an all-day absence exists for employee on a day
- **WHEN** composing cell items
- **THEN** the absence appears in the composed list

#### Scenario: Holiday items included
- **GIVEN** a German holiday exists for a day
- **WHEN** composing cell items
- **THEN** the holiday appears with German name
- **AND** it is marked as read-only

#### Scenario: Assignment items included
- **GIVEN** project assignments exist for employee on a day
- **WHEN** composing cell items
- **THEN** each assignment appears in the composed list

#### Scenario: Preexisting appointments included
- **GIVEN** preexisting appointments exist for employee on a day
- **WHEN** composing cell items
- **THEN** each appointment appears with title and start time

### Requirement: Read-only flag
The system SHALL flag non-editable items appropriately.

#### Scenario: Absences are read-only
- **GIVEN** an absence item in composition
- **WHEN** the item is created
- **THEN** isReadOnly is true

#### Scenario: Holidays are read-only
- **GIVEN** a holiday item in composition
- **WHEN** the item is created
- **THEN** isReadOnly is true

#### Scenario: Appointments are read-only
- **GIVEN** an appointment item in composition
- **WHEN** the item is created
- **THEN** isReadOnly is true

#### Scenario: Assignments are editable
- **GIVEN** an assignment item in composition
- **WHEN** the item is created
- **THEN** isReadOnly is false

### Requirement: Item ordering
The system SHALL sort items by type then start time.

#### Scenario: Items sorted by type
- **GIVEN** mixed item types for a day
- **WHEN** composing cell items
- **THEN** absences appear first, then holidays, then appointments, then assignments

#### Scenario: Assignments sorted by start time
- **GIVEN** multiple assignments with different start times
- **WHEN** composing cell items
- **THEN** assignments are sorted by start time ascending