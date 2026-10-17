# Spec Delta

## MODIFIED Requirements

### Requirement: Create new assignment
The system SHALL allow assigning a project or a custom appointment to an employee/day.

#### Scenario: Save new assignment
- **WHEN** user selects a project and clicks save
- **THEN** assignment is persisted
- **AND** modal closes
- **AND** the employee's row is refreshed without the loading spinner and shows the new assignment

#### Scenario: Save new custom appointment
- **WHEN** user selects "Eigener Termin", enters a title and clicks save
- **THEN** a custom appointment is persisted
- **AND** modal closes
- **AND** the employee's row is refreshed without the loading spinner and shows the new custom appointment

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
