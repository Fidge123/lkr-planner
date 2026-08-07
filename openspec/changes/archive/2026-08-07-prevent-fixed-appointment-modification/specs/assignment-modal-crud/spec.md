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
- **AND** a German notice explains the appointment is fixed and cannot be edited

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
- **AND** a German notice explains the appointment is fixed and cannot be removed

#### Scenario: Backend rejects a stale delete attempt
- **WHEN** a delete is submitted for an assignment that the backend determines is protected
- **THEN** the German error message returned by the backend is shown
- **AND** the assignment is not removed
