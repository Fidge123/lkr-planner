## ADDED Requirements

### Requirement: Default suggestions on modal open
The system SHALL show deterministic default suggestions when assignment modal opens.

#### Scenario: Show recent project first
- **WHEN** modal opens
- **THEN** first suggestion is the most recently assigned project
- **AND** suggestions are deterministic for same state

#### Scenario: Show overdue projects
- **WHEN** modal opens
- **THEN** show up to 4 overdue projects after the recent project
- **AND** total suggestions capped at 5

#### Scenario: Empty state handling
- **WHEN** no recent assignment AND no overdue projects exist
- **THEN** show German message "Keine Vorschläge verfügbar"
- **AND** modal allows free-text search

### Requirement: Suggestion count limit
The system SHALL cap total suggestions at 5.

#### Scenario: Cap suggestions at 5
- **WHEN** more than 5 suggestions available
- **THEN** return exactly 5 suggestions
- **AND** ordering follows: recent first, then overdue