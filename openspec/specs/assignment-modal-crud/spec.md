## Purpose

Give planners one modal to create, edit, and delete the assignment of an employee/day cell, and keep the weekly grid in sync after every save or delete.

## Requirements

### Requirement: Open modal from cell interaction
The system SHALL open the assignment modal in create mode when the user clicks the add affordance of an employee/day cell, and in edit mode when the user triggers an assignment card's edit action.

#### Scenario: Click empty cell opens create mode
- **WHEN** user clicks the add affordance on an employee/day cell without assignment
- **THEN** modal opens in create mode
- **AND** employee and day are pre-selected

#### Scenario: Click assigned cell opens edit mode
- **WHEN** the user triggers the edit action on an assignment card in an employee/day cell
- **THEN** modal opens in edit mode for that assignment
- **AND** current assignment is pre-populated

#### Scenario: Clicking an assignment card opens nothing
- **WHEN** the user clicks an assignment card anywhere other than its action controls
- **THEN** no modal opens
- **AND** the modal is reached through the card's edit action instead

### Requirement: Create new assignment
The system SHALL allow assigning a project to an employee/day.

#### Scenario: Save new assignment
- **WHEN** user selects a project and clicks save
- **THEN** assignment is persisted
- **AND** modal closes
- **AND** weekly grid shows new assignment immediately

### Requirement: Edit existing assignment
The system SHALL allow editing an existing assignment, unless it is protected because its linked Daylite project has category `"Termin FIX geplant"` and the user has not unlocked it.

#### Scenario: Change project
- **WHEN** user changes the project in edit mode and saves a non-protected assignment
- **THEN** assignment is updated
- **AND** grid reflects change immediately

#### Scenario: Edit controls disabled for protected assignment
- **WHEN** modal opens in edit mode for an assignment whose linked project has category `"Termin FIX geplant"`
- **THEN** the save control is disabled
- **AND** a German notice explains the appointment is fixed and points to the unlock control

#### Scenario: Backend rejects a stale edit attempt
- **WHEN** a save is submitted for an assignment that the backend determines is protected
- **THEN** the German error message returned by the backend is shown
- **AND** the assignment is not modified

### Requirement: Delete assignment
The system SHALL allow removing an assignment, unless it is protected because its linked Daylite project has category `"Termin FIX geplant"` and the user has not unlocked it.

#### Scenario: Delete assignment
- **WHEN** user clicks delete and confirms for a non-protected assignment
- **THEN** assignment is removed
- **AND** grid updates to show empty cell

#### Scenario: Delete control disabled for protected assignment
- **WHEN** modal opens in edit mode for an assignment whose linked project has category `"Termin FIX geplant"`
- **THEN** the delete control is disabled
- **AND** a German notice explains the appointment is fixed and points to the unlock control

#### Scenario: Backend rejects a stale delete attempt
- **WHEN** a delete is submitted for an assignment that the backend determines is protected
- **THEN** the German error message returned by the backend is shown
- **AND** the assignment is not removed

### Requirement: Unsaved changes handling
The system SHALL handle unsaved changes properly.

#### Scenario: Close with unsaved changes
- **WHEN** user tries to close modal with unsaved changes
- **THEN** confirmation dialog appears
- **AND** user can save, discard, or cancel

### Requirement: Grid reload after save
The system SHALL refresh only the edited employee's row of the displayed week after a successful save or delete, without the loading spinner.

#### Scenario: Row refreshes after save
- **WHEN** assignment is saved successfully
- **THEN** the modal closes
- **AND** the employee's calendar events of the displayed week are fetched again and replace that row without the loading spinner
- **AND** the new or updated assignment is visible

#### Scenario: Card is removed after delete
- **WHEN** assignment is deleted successfully
- **THEN** the modal closes and the card is removed from the grid immediately
- **AND** the employee's row is then refreshed without the loading spinner

#### Scenario: Failed row refresh
- **WHEN** the refresh of the employee's row fails
- **THEN** the whole displayed week is loaded again

### Requirement: Project picker
The system SHALL provide a project picker in the modal for selecting a project.

#### Scenario: Picker shows active projects only
- **WHEN** assignment modal is open
- **THEN** project picker lists only projects with status `new_status` or `in_progress`

#### Scenario: Picker is pre-populated in edit mode
- **WHEN** modal opens in edit mode
- **THEN** the currently assigned project is pre-selected in the picker

### Requirement: Unlock a protected assignment
The system SHALL offer an unlock control on the assignment modal's fixed-appointment notice that re-enables the disabled affordances for the current modal session.

#### Scenario: Notice offers the unlock control
- **WHEN** the modal opens in edit mode for an assignment whose linked project has category `"Termin FIX geplant"`
- **THEN** the German notice carries an unlock control
- **AND** the control is off
- **AND** the project picker, the save control and the delete control are disabled

#### Scenario: Unlocking re-enables the controls
- **WHEN** the user switches the unlock control on
- **THEN** the project picker, the save control and the delete control become usable
- **AND** saving or deleting writes through with the protection override set

#### Scenario: Unlock is not remembered
- **WHEN** the modal is closed and reopened for the same assignment
- **THEN** the unlock control is off again
- **AND** the controls are disabled again

#### Scenario: Unprotected assignment shows no unlock control
- **WHEN** the modal opens in edit mode for an assignment that is not protected
- **THEN** no notice and no unlock control are shown
