# Spec Delta

## MODIFIED Requirements

### Requirement: Next-day translucent ghost after create
The system SHALL show a single translucent ghost on the next visible day of the current week after an assignment is created.
The ghost carries the project, or the custom appointment title, that was just created for the same employee.
Only creating an assignment or a custom appointment triggers a ghost.
Editing or deleting an assignment SHALL NOT trigger a ghost.

#### Scenario: Ghost after creating an assignment
- **WHEN** the user creates an assignment for employee X on day Y
- **AND** day Y is not the last visible day of the current week
- **THEN** a translucent ghost of the created project appears for employee X on the next visible day
- **AND** the ghost is visually distinct from persisted assignments

#### Scenario: Ghost after creating a custom appointment
- **WHEN** the user creates a custom appointment for employee X on day Y
- **AND** day Y is not the last visible day of the current week
- **THEN** a translucent ghost showing the custom appointment's title appears for employee X on the next visible day

#### Scenario: No ghost after editing an assignment
- **WHEN** the user edits an existing assignment
- **THEN** no ghost is shown

#### Scenario: No ghost on the last visible day
- **WHEN** the user creates an assignment on the last visible day of the current week
- **THEN** no ghost is shown because the next visible day is outside the current week

#### Scenario: Ghost suppressed when target day is occupied
- **WHEN** creating an assignment would place a ghost on a day that already holds any event for that employee
- **THEN** no ghost is shown for that day

### Requirement: One-click add with chaining
The system SHALL create a persisted assignment, or a custom appointment with the same title, when the user clicks a ghost.
Because a click creates an assignment, it SHALL in turn produce the next ghost one visible day further, enabling day-by-day chaining.

#### Scenario: Click creates a persisted assignment
- **WHEN** the user clicks a ghost on day Y
- **THEN** a persisted assignment for the ghost's project is created for that employee on day Y

#### Scenario: Click on a custom appointment ghost
- **WHEN** the user clicks a custom appointment ghost on day Y
- **THEN** a custom appointment with the ghost's title is created for that employee on day Y

#### Scenario: Chaining to the following day
- **WHEN** the user clicks a ghost on day Y
- **AND** day Y is not the last visible day of the current week
- **AND** the following visible day holds no event for that employee
- **THEN** a new ghost of the same project or custom title appears on the following visible day

#### Scenario: Chain ends at the last visible day
- **WHEN** the user clicks a ghost on the last visible day of the current week
- **THEN** the assignment is created
- **AND** no further ghost is shown
