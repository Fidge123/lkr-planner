## ADDED Requirements

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
