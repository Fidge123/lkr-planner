## ADDED Requirements

### Requirement: Live project filter
The system SHALL filter projects as user types in the filter input.
The result list starts empty; default content for the empty state is supplied by the `assignment-modal-suggestions` capability and is out of scope here.

#### Scenario: Typing shows filtered results
- **WHEN** user types at least 3 characters in the filter input
- **THEN** filtered results are shown in the result list
- **AND** only projects with status `new_status` or `in_progress` are included

#### Scenario: Short filter does not query
- **WHEN** the filter input holds fewer than 3 characters
- **THEN** no filter query is sent
- **AND** the result list stays in its empty default state

#### Scenario: Filtered results limited to 5
- **WHEN** user filters projects
- **THEN** maximum 5 matching projects are shown
- **AND** results are sorted by project name

#### Scenario: Clear filter empties the result list
- **WHEN** user clears the filter input
- **THEN** filtered results are hidden
- **AND** the result list returns to its empty default state

### Requirement: Keyboard navigation
The system SHALL support keyboard selection over whichever result list is currently displayed (filtered results or the empty-state default content).

#### Scenario: Arrow key navigation
- **WHEN** a result list is shown and user presses Arrow Down
- **THEN** the next item is highlighted
- **AND** pressing Arrow Up highlights the previous item

#### Scenario: Enter to select
- **WHEN** an item is highlighted and user presses Enter
- **THEN** that project is selected into the assignment field
- **AND** the modal stays open so the user confirms with the Speichern button

#### Scenario: Escape clears a non-empty filter
- **WHEN** the filter input is non-empty and user presses Escape
- **THEN** the filter input is cleared
- **AND** the result list returns to its empty default state
- **AND** the modal does not close

#### Scenario: Escape closes the modal when filter is empty
- **WHEN** the filter input is empty and user presses Escape
- **THEN** the modal close flow runs (existing unsaved-changes guard applies)