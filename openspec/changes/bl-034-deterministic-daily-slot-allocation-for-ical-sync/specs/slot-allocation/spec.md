## ADDED Requirements

### Requirement: Deterministic slot allocation
The system SHALL allocate time slots for same-day assignments in a deterministic manner.

#### Scenario: Single assignment slot
- **GIVEN** 1 assignment for a day
- **WHEN** allocating time slots
- **THEN** the assignment receives the full 08:00-16:00 window

#### Scenario: Two equal assignments
- **GIVEN** 2 assignments for a day
- **WHEN** allocating time slots
- **THEN** assignment 1 receives 08:00-12:00
- **AND** assignment 2 receives 12:00-16:00

#### Scenario: Three equal assignments
- **GIVEN** 3 assignments for a day
- **WHEN** allocating time slots
- **THEN** assignment 1 receives 08:00-10:40
- **AND** assignment 2 receives 10:40-13:20
- **AND** assignment 3 receives 13:20-16:00

#### Scenario: Reordered input produces same output
- **GIVEN** assignments A, B in order [A, B]
- **AND** same assignments A, B in order [B, A]
- **WHEN** allocating time slots for both orderings
- **THEN** both produce identical slot assignments

### Requirement: Non-overlapping slots
The system SHALL ensure allocated slots do not overlap.

#### Scenario: No slot overlap
- **GIVEN** n assignments for a day
- **WHEN** allocating time slots
- **THEN** each slot's end time equals next slot's start time
- **AND** no time is double-allocated

### Requirement: Fixed time window
The system SHALL use fixed 08:00-16:00 window.

#### Scenario: Window boundaries
- **GIVEN** any number of assignments
- **WHEN** allocating time slots
- **THEN** first slot starts at 08:00
- **AND** last slot ends at 16:00