## Purpose

Write assignment creates, updates, and deletes through to the employee's primary CalDAV calendar idempotently, and never write into an absence calendar.

## Requirements

### Requirement: Create assignment in CalDAV
The system SHALL write a new VEVENT to the employee's primary CalDAV calendar when an assignment is created.

#### Scenario: Create assignment triggers CalDAV PUT
- **WHEN** a new assignment is created via the modal (BL-016)
- **THEN** a VEVENT is written to the employee's primary CalDAV calendar
- **AND** the event contains the Daylite project reference in DESCRIPTION first line
- **AND** the event has a stable UUID as its UID

#### Scenario: Create fails — CalDAV unavailable
- **WHEN** CalDAV write fails (server unreachable, auth error)
- **THEN** a German error message is shown to the user
- **AND** the user can retry from the modal

### Requirement: Update assignment in CalDAV
The system SHALL update an existing VEVENT when an assignment is modified, unless the event is protected (see `fixed-appointment-protection`).

#### Scenario: Update assignment triggers CalDAV PUT
- **WHEN** an existing, non-protected assignment is modified
- **THEN** the existing VEVENT is updated via PUT using the stable UID
- **AND** the UID does not change
- **AND** repeated updates produce exactly one event

#### Scenario: Update blocked for protected assignment
- **WHEN** an update is attempted on an assignment whose linked Daylite project has category `"Termin FIX geplant"`
- **THEN** the update is rejected before any CalDAV PUT request
- **AND** a German error message is returned

### Requirement: Delete assignment from CalDAV
The system SHALL remove a VEVENT when an assignment is deleted, unless the event is protected (see `fixed-appointment-protection`).

#### Scenario: Delete assignment triggers CalDAV DELETE
- **WHEN** a non-protected assignment is deleted
- **THEN** the corresponding VEVENT is removed from CalDAV
- **AND** the operation is idempotent (no error if the event is already absent)

#### Scenario: Delete blocked for protected assignment
- **WHEN** a delete is attempted on an assignment whose linked Daylite project has category `"Termin FIX geplant"`
- **THEN** the delete is rejected before any CalDAV DELETE request
- **AND** a German error message is returned

### Requirement: Idempotent writes
The system SHALL ensure repeated write operations do not produce duplicate events.

#### Scenario: Repeated PUT with same UID
- **WHEN** a create or update is retried with the same assignment
- **THEN** exactly one VEVENT exists in CalDAV for that assignment
- **AND** the event reflects the latest state

### Requirement: Absence calendar is never written
The system SHALL never perform write operations against absence calendar URLs.

#### Scenario: Write blocked for absence calendar URL
- **WHEN** a write operation is attempted
- **AND** the target URL matches the employee's absence calendar URL
- **THEN** the operation is rejected before any network request
- **AND** the incident is logged
