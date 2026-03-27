## ADDED Requirements

### Requirement: Open modal from cell interaction
The system SHALL open the assignment modal when user clicks an employee/day cell.

#### Scenario: Click empty cell opens create mode
- **WHEN** user clicks on an employee/day cell without assignment
- **THEN** modal opens in create mode
- **AND** employee and day are pre-selected

#### Scenario: Click assigned cell opens edit mode
- **WHEN** user clicks on an employee/day cell with existing assignment
- **THEN** modal opens in edit mode
- **AND** current assignment is pre-populated

### Requirement: Create new assignment
The system SHALL allow assigning a project to an employee/day.

#### Scenario: Save new assignment
- **WHEN** user selects a project and clicks save
- **THEN** assignment is persisted
- **AND** modal closes
- **AND** weekly grid shows new assignment immediately

### Requirement: Edit existing assignment
The system SHALL allow editing an existing assignment.

#### Scenario: Change project
- **WHEN** user changes the project in edit mode and saves
- **THEN** assignment is updated
- **AND** grid reflects change immediately

### Requirement: Delete assignment
The system SHALL allow removing an assignment.

#### Scenario: Delete assignment
- **WHEN** user clicks delete and confirms
- **THEN** assignment is removed
- **AND** grid updates to show empty cell

### Requirement: Unsaved changes handling
The system SHALL handle unsaved changes properly.

#### Scenario: Close with unsaved changes
- **WHEN** user tries to close modal with unsaved changes
- **THEN** confirmation dialog appears
- **AND** user can save, discard, or cancel

### Requirement: Immediate UI update
The system SHALL update the weekly grid immediately after save.

#### Scenario: Grid updates after save
- **WHEN** assignment is saved
- **THEN** weekly grid updates without page reload