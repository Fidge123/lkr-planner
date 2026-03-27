## ADDED Requirements

### Requirement: Load assignments from persistence
The system SHALL load assignment state from persistent storage.

#### Scenario: Load assignments for week
- **WHEN** user navigates to a week
- **THEN** assignments are loaded from persistent storage
- **AND** displayed in the planning grid

#### Scenario: No assignments exist
- **WHEN** loading assignments for a week with no saved data
- **THEN** empty state is shown in German
- **AND** user can create new assignments

### Requirement: Save assignments to persistence
The system SHALL save assignment state to persistent storage.

#### Scenario: Persist new assignment
- **WHEN** user creates an assignment
- **THEN** the assignment is saved to persistent storage
- **AND** survives app restart

#### Scenario: Update existing assignment
- **WHEN** user modifies an assignment
- **THEN** the updated assignment replaces the old one
- **AND** change persists across restarts

#### Scenario: Delete assignment
- **WHEN** user removes an assignment
- **THEN** the assignment is removed from storage
- **AND** removal persists across restarts

### Requirement: Week navigation with persistence
The system SHALL use the same persisted source for all week navigation.

#### Scenario: Navigate between weeks
- **WHEN** user navigates to different weeks
- **THEN** each week's assignments are loaded from persistent storage
- **AND** data is consistent across navigation

### Requirement: Loading and error states
The system SHALL maintain German loading and error states.

#### Scenario: Show loading state
- **WHEN** assignments are being loaded
- **THEN** German loading indicator is shown

#### Scenario: Show error state on load failure
- **WHEN** loading assignments fails
- **THEN** German error message is displayed
- **AND** user can retry