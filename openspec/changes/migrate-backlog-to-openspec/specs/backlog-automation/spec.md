## ADDED Requirements

### Requirement: Openspec is the standard change workflow
The system SHALL use openspec as the standard workflow for all new changes.

#### Scenario: New changes use openspec
- **WHEN** a new feature or bug fix is proposed
- **THEN** an openspec change is created using `openspec new change`
- **AND** the proposal-design-tasks workflow is followed

### Requirement: Backlog items are tracked in openspec
The system SHALL track all pending work in openspec rather than the old markdown backlog.

#### Scenario: New backlog items go to openspec
- **WHEN** a new backlog item is identified
- **THEN** it is created directly as an openspec change
- **AND** no new files are added to docs/backlog/

### Requirement: Completed changes are archived
The system SHALL provide a process to archive completed changes.

#### Scenario: Completed changes are archived
- **WHEN** an openspec change is completed
- **THEN** it can be archived using the openspec archive workflow
- **AND** the archive preserves the change history
