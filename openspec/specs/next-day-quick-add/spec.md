## Purpose

Speed up entering a run of consecutive daily assignments for the same employee and project by showing a translucent "ghost" suggestion on the next visible day after an assignment is created, which the user can accept with a single click.

## Requirements

### Requirement: Next-day translucent ghost after create
The system SHALL show a single translucent ghost on the next visible day of the current week after an assignment is created.
The ghost carries the project that was just created for the same employee.
Only creating an assignment triggers a ghost.
Editing or deleting an assignment SHALL NOT trigger a ghost.

#### Scenario: Ghost after creating an assignment
- **WHEN** the user creates an assignment for employee X on day Y
- **AND** day Y is not the last visible day of the current week
- **THEN** a translucent ghost of the created project appears for employee X on the next visible day
- **AND** the ghost is visually distinct from persisted assignments

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
The system SHALL create a persisted assignment when the user clicks a ghost.
Because a click creates an assignment, it SHALL in turn produce the next ghost one visible day further, enabling day-by-day chaining.

#### Scenario: Click creates a persisted assignment
- **WHEN** the user clicks a ghost on day Y
- **THEN** a persisted assignment for the ghost's project is created for that employee on day Y

#### Scenario: Chaining to the following day
- **WHEN** the user clicks a ghost on day Y
- **AND** day Y is not the last visible day of the current week
- **AND** the following visible day holds no event for that employee
- **THEN** a new ghost of the same project appears on the following visible day

#### Scenario: Chain ends at the last visible day
- **WHEN** the user clicks a ghost on the last visible day of the current week
- **THEN** the assignment is created
- **AND** no further ghost is shown

### Requirement: Ghost clearing
The system SHALL clear the ghost when the user deletes any assignment or navigates to another week.
Cancelling a modal without saving SHALL NOT clear the ghost.

#### Scenario: Any delete clears the ghost
- **WHEN** a ghost is shown
- **AND** the user deletes any assignment
- **THEN** the ghost is removed

#### Scenario: Week navigation clears the ghost
- **WHEN** a ghost is shown
- **AND** the user navigates to another week
- **THEN** the ghost is removed

#### Scenario: Cancel keeps the ghost
- **WHEN** a ghost is shown
- **AND** the user opens a modal and closes it without saving
- **THEN** the ghost remains

### Requirement: Visual distinction
The system SHALL clearly distinguish a ghost from persisted assignments.

#### Scenario: Ghost styling
- **WHEN** a ghost is displayed
- **THEN** it has reduced opacity (~50%)
- **AND** it has a dashed border
