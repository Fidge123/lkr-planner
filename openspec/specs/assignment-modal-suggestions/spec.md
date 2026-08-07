## Purpose

Fill the assignment modal's empty result list with deterministic default suggestions, the most recently assigned project followed by overdue projects, so the common case needs no typing.

## Requirements

### Requirement: Default suggestions on modal open
The system SHALL show deterministic default suggestions when assignment modal opens.

#### Scenario: Show recent project first
- **WHEN** modal opens
- **AND** the client last-used cache holds a project from an earlier assignment this session
- **THEN** first suggestion is that most recently assigned project
- **AND** suggestions are deterministic for the same cache and overdue state

#### Scenario: Show overdue projects after a recent project
- **WHEN** modal opens
- **AND** a recent project exists in the client last-used cache
- **THEN** show up to 4 overdue projects after the recent project
- **AND** overdue projects are those with Daylite category `"Überfällig"` and status `new_status` or `in_progress`
- **AND** total suggestions capped at 5

#### Scenario: Show overdue projects with no recent project
- **WHEN** modal opens
- **AND** the client last-used cache is empty
- **THEN** show up to 5 overdue projects
- **AND** overdue projects are those with Daylite category `"Überfällig"` and status `new_status` or `in_progress`

#### Scenario: Recent project that is also overdue appears once
- **WHEN** modal opens
- **AND** the recent project is also in the overdue results
- **THEN** the project appears only once, in the recent (first) position
- **AND** it is removed from the overdue portion of the list

#### Scenario: Empty state handling
- **WHEN** the client last-used cache is empty AND no overdue projects exist
- **THEN** show German message "Keine Vorschläge verfügbar"
- **AND** modal allows free-text search

### Requirement: Default suggestions fill the combobox empty state
The system SHALL render default suggestions as the combobox empty-state content, including when the filter is cleared or Escape resets a non-empty filter.

#### Scenario: Suggestions restored when filter is cleared
- **WHEN** the filter input is cleared (manually or via Escape on a non-empty filter)
- **THEN** the result list returns to its empty default state
- **AND** the default suggestions (recent + overdue) are shown again

#### Scenario: Keyboard navigation over suggestions
- **WHEN** the default suggestions are displayed and the user presses Arrow Down / Arrow Up
- **THEN** the highlighted suggestion moves accordingly
- **AND** pressing Enter selects the highlighted suggestion into the assignment field
- **AND** the modal stays open so the user confirms with the Speichern button

### Requirement: Reserved space while suggestions load
The system SHALL reserve the height of a full suggestion list while the default suggestions are loading, so the dialog does not resize when they arrive.

#### Scenario: Modal opens before the suggestions arrive
- **WHEN** the modal opens and the filter is empty
- **AND** the default suggestions have not loaded yet
- **THEN** five placeholder rows are shown in place of the result list
- **AND** each placeholder row has the height of a loaded result row

#### Scenario: Placeholder replaced by the suggestions
- **WHEN** the default suggestions have loaded
- **THEN** the placeholder rows are replaced by the result list

### Requirement: Suggestion count limit
The system SHALL cap total suggestions at 5.

#### Scenario: Cap suggestions at 5
- **WHEN** more than 5 distinct suggestions available
- **THEN** return exactly 5 suggestions
- **AND** ordering follows: recent first (if any), then overdue
- **AND** a project never appears twice across the recent and overdue portions
