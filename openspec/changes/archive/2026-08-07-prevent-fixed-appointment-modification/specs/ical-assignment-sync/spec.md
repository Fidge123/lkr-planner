## MODIFIED Requirements

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
