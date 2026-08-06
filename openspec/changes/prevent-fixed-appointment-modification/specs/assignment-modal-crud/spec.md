## MODIFIED Requirements

### Requirement: Edit existing assignment
The system SHALL allow editing an existing assignment, unless it is protected because its linked Daylite project has category `"Termin FIX geplant"`.

#### Scenario: Change project
- **WHEN** user changes the project in edit mode and saves a non-protected assignment
- **THEN** assignment is updated
- **AND** grid reflects change immediately

#### Scenario: Edit controls disabled for protected assignment
- **WHEN** modal opens in edit mode for an assignment whose linked project has category `"Termin FIX geplant"`
- **THEN** the save control is disabled
- **AND** the project picker is disabled
- **AND** a German notice explains the appointment is fixed and offers an unlock checkbox

#### Scenario: Unlock checkbox re-enables editing
- **WHEN** the user ticks the unlock checkbox on a protected assignment
- **THEN** the project picker and the save control become editable
- **AND** saving writes the change through, overriding the protection

#### Scenario: Unlock is not remembered
- **WHEN** the modal is reopened after an assignment was unlocked
- **THEN** the unlock checkbox is unticked again
- **AND** the controls are disabled again

#### Scenario: Backend rejects a stale edit attempt
- **WHEN** a save is submitted for an assignment that the backend determines is protected
- **THEN** the German error message returned by the backend is shown
- **AND** the assignment is not modified

### Requirement: Delete assignment
The system SHALL allow removing an assignment, unless it is protected because its linked Daylite project has category `"Termin FIX geplant"`.

#### Scenario: Delete assignment
- **WHEN** user clicks delete and confirms for a non-protected assignment
- **THEN** assignment is removed
- **AND** grid updates to show empty cell

#### Scenario: Delete control disabled for protected assignment
- **WHEN** modal opens in edit mode for an assignment whose linked project has category `"Termin FIX geplant"`
- **THEN** the delete control is disabled
- **AND** a German notice explains the appointment is fixed and offers an unlock checkbox

#### Scenario: Unlock checkbox re-enables deletion
- **WHEN** the user ticks the unlock checkbox on a protected assignment
- **THEN** the delete control becomes usable
- **AND** confirming the deletion removes the assignment, overriding the protection

#### Scenario: Backend rejects a stale delete attempt
- **WHEN** a delete is submitted for an assignment that the backend determines is protected
- **THEN** the German error message returned by the backend is shown
- **AND** the assignment is not removed
