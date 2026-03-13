## ADDED Requirements

### Requirement: Render all item types
The system SHALL render all composed calendar cell items.

#### Scenario: Render absence items
- **GIVEN** an absence item in composed list
- **WHEN** rendering calendar cell
- **THEN** the absence displays with "Abwesenheit" label
- **AND** it is non-interactive (read-only)

#### Scenario: Render holiday items
- **GIVEN** a holiday item in composed list
- **WHEN** rendering calendar cell
- **THEN** the holiday displays with German name
- **AND** it is non-interactive (read-only)

#### Scenario: Render assignment items
- **GIVEN** an assignment item in composed list
- **WHEN** rendering calendar cell
- **THEN** the assignment displays with project title
- **AND** start/end time are shown
- **AND** clicking opens edit modal

#### Scenario: Render appointment items
- **GIVEN** an appointment item in composed list
- **WHEN** rendering calendar cell
- **THEN** the appointment displays with title
- **AND** start/end time are shown
- **AND** it is non-interactive (read-only)

### Requirement: Project title fallback
The system SHALL determine project title using fallback order.

#### Scenario: Custom name used
- **GIVEN** an assignment with custom name set
- **WHEN** determining display title
- **THEN** the custom name is shown

#### Scenario: Planradar fallback
- **GIVEN** an assignment with no custom name but Planradar project name
- **WHEN** determining display title
- **THEN** the Planradar project name is shown

#### Scenario: Single Daylite company fallback
- **GIVEN** an assignment with no custom name, no Planradar, but exactly one linked company
- **WHEN** determining display title
- **THEN** the Daylite company name is shown

#### Scenario: Daylite project fallback
- **GIVEN** an assignment with no custom name, no Planradar, and zero or multiple linked companies
- **WHEN** determining display title
- **THEN** the Daylite project name is shown

### Requirement: Time display
The system SHALL display start and end times for items with times.

#### Scenario: Assignment times
- **GIVEN** an assignment with start and end time
- **WHEN** rendering the item
- **THEN** both times are displayed

#### Scenario: All-day items
- **GIVEN** an absence or holiday item
- **WHEN** rendering the item
- **THEN** an all-day indicator is shown instead of times