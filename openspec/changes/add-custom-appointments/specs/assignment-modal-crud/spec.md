# Spec Delta

## MODIFIED Requirements

### Requirement: Create new assignment
The system SHALL allow assigning a project or a custom appointment to an employee/day.

#### Scenario: Save new assignment
- **WHEN** user selects a project and clicks save
- **THEN** assignment is persisted
- **AND** modal closes
- **AND** weekly grid shows new assignment immediately

#### Scenario: Save new custom appointment
- **WHEN** user selects "Eigener Termin", enters a title and clicks save
- **THEN** a custom appointment is persisted
- **AND** modal closes
- **AND** weekly grid shows the new custom appointment immediately

### Requirement: Project picker
The system SHALL provide a project picker in the modal for selecting a project while the appointment type is "Projekt".

#### Scenario: Picker shows active projects only
- **WHEN** assignment modal is open
- **THEN** project picker lists only projects with status `new_status` or `in_progress`

#### Scenario: Picker is pre-populated in edit mode
- **WHEN** modal opens in edit mode
- **THEN** the currently assigned project is pre-selected in the picker

#### Scenario: Picker hidden for custom appointments
- **WHEN** the appointment type is "Eigener Termin"
- **THEN** the project picker is not shown

### Requirement: Unlock a protected assignment
The system SHALL offer an unlock control on the assignment modal's fixed-appointment notice that re-enables the disabled affordances for the current modal session.

#### Scenario: Notice offers the unlock control
- **WHEN** the modal opens in edit mode for an assignment whose linked project has category `"Termin FIX geplant"`
- **THEN** the German notice carries an unlock control
- **AND** the control is off
- **AND** the appointment type switch, the project picker, the save control and the delete control are disabled

#### Scenario: Unlocking re-enables the controls
- **WHEN** the user switches the unlock control on
- **THEN** the appointment type switch, the project picker, the save control and the delete control become usable
- **AND** saving or deleting writes through with the protection override set

#### Scenario: Unlock is not remembered
- **WHEN** the modal is closed and reopened for the same assignment
- **THEN** the unlock control is off again
- **AND** the controls are disabled again

#### Scenario: Unprotected assignment shows no unlock control
- **WHEN** the modal opens in edit mode for an assignment that is not protected
- **THEN** no notice and no unlock control are shown
