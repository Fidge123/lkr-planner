## ADDED Requirements

### Requirement: Migrate pending backlog items to openspec
The system SHALL provide a process to migrate all pending backlog items from the markdown-based system in `docs/backlog/` to openspec changes.

#### Scenario: Migration creates valid openspec changes
- **WHEN** a backlog item is migrated using the migration process
- **THEN** an openspec change is created with proposal, design, and tasks artifacts
- **AND** the change is in apply-ready state

#### Scenario: Each backlog item becomes one change
- **WHEN** multiple backlog items exist in the old system
- **THEN** each item creates a separate openspec change
- **AND** the change name uses the BLI identifier

### Requirement: Preserve backlog item context
The migration SHALL preserve the essential context from each backlog item including the description, and acceptance criteria.

#### Scenario: Original content is preserved
- **WHEN** a backlog item is migrated
- **THEN** the proposal.md contains the original description
- **AND** any acceptance criteria are reflected in tasks

### Requirement: Backlog items are traceable
The system SHALL maintain traceability between old and new systems.

#### Scenario: BLI identifiers are preserved
- **WHEN** a backlog item with ID BLI-XXX is migrated
- **THEN** the openspec change uses BLI-XXX as part of the name
- **AND** the original file reference is documented
