## MODIFIED Requirements

### Requirement: Absence event display
The system SHALL render absence events visually distinct from assignment and bare events in the timetable, using a color derived from the absence category.

#### Scenario: Absence cell styling
- **WHEN** a timetable cell contains an absence event
- **THEN** the absence event is rendered non-interactively (no button)
- **AND** it uses a color derived from its absence category, distinct from assignment and bare event colors

#### Scenario: Sick absence category
- **WHEN** an absence event's title contains "krank" (case-insensitive)
- **THEN** the absence event uses the sick category color (`bg-error/30`)

#### Scenario: Special leave or training absence category
- **WHEN** an absence event's title contains "sonderurlaub", "fortbildung", or "schulung" (case-insensitive)
- **THEN** the absence event uses the special leave/training category color (`bg-accent/30`)

#### Scenario: Vacation or unmatched absence category
- **WHEN** an absence event's title does not match any known category keyword
- **THEN** the absence event uses the default absence color (`bg-info/30`)

#### Scenario: Multiple event types in same cell
- **WHEN** a cell contains both an absence event and an assignment event
- **THEN** both are rendered in the cell
- **AND** each uses its respective styling
