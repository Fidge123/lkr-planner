## ADDED Requirements

### Requirement: Next-day translucent suggestion
The system SHALL show a translucent suggestion in the next day after an assignment is saved.

#### Scenario: Show suggestion after assignment save
- **WHEN** user saves an assignment for employee X on day Y
- **THEN** a translucent copy appears in day Y+1 for employee X
- **AND** the suggestion is visually distinct from persisted assignments

#### Scenario: One-click add from suggestion
- **WHEN** user clicks on the translucent suggestion
- **THEN** a real assignment is created for next day
- **AND** the suggestion is replaced with persisted assignment

#### Scenario: Remove suggestion when source deleted
- **WHEN** user deletes the source assignment
- **THEN** the translucent suggestion is removed

### Requirement: Visual distinction
The system SHALL clearly distinguish suggestions from persisted assignments.

#### Scenario: Suggestion styling
- **WHEN** a suggestion is displayed
- **THEN** it has reduced opacity (~50%)
- **AND** it has a dashed border
