## MODIFIED Requirements

### Requirement: Absence event display
The system SHALL render absence events visually distinct from assignment and bare events in the timetable, using a color derived from the absence code.

#### Scenario: Absence cell styling
- **WHEN** a timetable cell contains an absence event
- **THEN** the absence event is rendered non-interactively (no button)
- **AND** it uses a color derived from its absence code, distinct from assignment and bare event colors

#### Scenario: Vacation family codes
- **WHEN** an absence event's title's leading code is `UB`, `SU`, or `UU` (case-insensitive)
- **THEN** the absence event uses the vacation hue, at a distinct intensity per code (`UB` strongest, `SU` medium, `UU` lightest)

#### Scenario: Sick family codes
- **WHEN** an absence event's title's leading code is `KR` or `Kro` (case-insensitive)
- **THEN** the absence event uses the sick hue, at a distinct intensity per code (`KR` stronger, `Kro` lighter)

#### Scenario: Time off in lieu code
- **WHEN** an absence event's title's leading code is `FA` (case-insensitive)
- **THEN** the absence event uses its own distinct hue, separate from the vacation and sick hues

#### Scenario: Unmatched absence code
- **WHEN** an absence event's title does not match any known code
- **THEN** the absence event uses the default absence color (`bg-info/30`)

#### Scenario: Multiple event types in same cell
- **WHEN** a cell contains both an absence event and an assignment event
- **THEN** both are rendered in the cell
- **AND** each uses its respective styling

### Requirement: Absence and appointment conflict indicator
The system SHALL highlight a timetable cell in red, with an icon and German label, when it contains both an absence event and an assignment event for the same employee and day.

#### Scenario: Absence and appointment coincide
- **WHEN** a timetable cell contains both an absence event and an assignment event for the same employee and day
- **THEN** the cell shows a red conflict indicator
- **AND** the indicator includes an icon and a German label explaining the conflict
- **AND** the absence and assignment events keep their own category/status colors alongside the indicator

#### Scenario: Absence without an appointment
- **WHEN** a timetable cell contains an absence event and no assignment event
- **THEN** no conflict indicator is shown

#### Scenario: Bare event does not trigger a conflict
- **WHEN** a timetable cell contains an absence event and a bare (non-assignment) event
- **THEN** no conflict indicator is shown
