## ADDED Requirements

### Requirement: Sync create operations
The system SHALL synchronize assignment creation to employee primary iCal.

#### Scenario: Create assignment triggers sync
- **WHEN** a new assignment is created
- **THEN** corresponding iCal event is created in employee's primary calendar
- **AND** event contains assignment details (project, time slot)

### Requirement: Sync update operations
The system SHALL synchronize assignment updates to employee primary iCal.

#### Scenario: Update assignment triggers sync
- **WHEN** an assignment is modified
- **THEN** existing iCal event is updated with new details
- **AND** event UID remains stable for idempotency

### Requirement: Sync delete operations
The system SHALL synchronize assignment deletion to employee primary iCal.

#### Scenario: Delete assignment triggers sync
- **WHEN** an assignment is deleted
- **THEN** corresponding iCal event is removed
- **AND** removal is idempotent (no error if event doesn't exist)

### Requirement: Idempotent sync
The system SHALL ensure sync operations are idempotent.

#### Scenario: Repeated sync produces no duplicates
- **WHEN** sync runs multiple times for same assignment
- **THEN** only one iCal event exists
- **AND** event reflects latest assignment state

### Requirement: Sync status tracking
The system SHALL track and expose sync status per assignment.

#### Scenario: Track sync status
- **WHEN** sync operation completes
- **THEN** status (pending/synced/failed) is stored
- **AND** status is accessible for troubleshooting

### Requirement: Absence calendar is read-only
The system SHALL never write to absence calendars.

#### Scenario: Absence calendar not modified
- **WHEN** sync operations run
- **THEN** no write operations target absence iCal URLs
- **AND** only primary assignment calendars are modified