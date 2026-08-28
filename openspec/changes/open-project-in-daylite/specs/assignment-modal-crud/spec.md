## MODIFIED Requirements

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
