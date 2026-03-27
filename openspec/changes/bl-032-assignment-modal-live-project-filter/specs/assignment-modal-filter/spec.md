## ADDED Requirements

### Requirement: Live project filter
The system SHALL filter projects as user types in the filter input.

#### Scenario: Filter replaces suggestions
- **WHEN** user types at least 1 character in filter input
- **THEN** default suggestions are hidden
- **AND** filtered results are shown instead
- **AND** only projects with status `new_status` or `in_progress` are included

#### Scenario: Filtered results limited to 5
- **WHEN** user filters projects
- **THEN** maximum 5 matching projects are shown
- **AND** results are sorted by project name

#### Scenario: Clear filter restores suggestions
- **WHEN** user clears the filter input
- **THEN** filtered results are hidden
- **AND** default suggestions are restored

### Requirement: Keyboard navigation
The system SHALL support keyboard selection in filtered list.

#### Scenario: Arrow key navigation
- **WHEN** filtered list is shown and user presses Arrow Down
- **THEN** next project is highlighted
- **AND** pressing Arrow Up highlights previous project

#### Scenario: Enter to select
- **WHEN** a project is highlighted and user presses Enter
- **THEN** that project is selected
- **AND** modal closes with selection

#### Scenario: Escape to clear filter
- **WHEN** user presses Escape
- **THEN** filter input is cleared
- **AND** default suggestions are restored