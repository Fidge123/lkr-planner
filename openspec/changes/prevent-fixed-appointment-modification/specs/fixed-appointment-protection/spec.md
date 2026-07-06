## ADDED Requirements

### Requirement: Identify protected events by Daylite project category
The system SHALL identify a CalDAV event as protected when the Daylite project it references has category `"Termin FIX geplant"`.

#### Scenario: Event linked to protected category
- **WHEN** an event's DESCRIPTION contains a `daylite:/<path>` reference
- **AND** the referenced Daylite project's `category` is `"Termin FIX geplant"`
- **THEN** the event is treated as protected

#### Scenario: Event linked to non-protected category
- **WHEN** an event's DESCRIPTION contains a `daylite:/<path>` reference
- **AND** the referenced Daylite project's `category` is not `"Termin FIX geplant"` (including `null`)
- **THEN** the event is not treated as protected

#### Scenario: Event has no Daylite project reference
- **WHEN** an event's DESCRIPTION contains no `daylite:/<path>` reference (bare event)
- **THEN** the event is protected

#### Scenario: Project lookup fails
- **WHEN** the Daylite project referenced by an event cannot be resolved (network error, project not found)
- **THEN** the event is treated as not protected
- **AND** a warning message is shown to the user
- **AND** the event has a warning icon
- **AND** the lookup failure is logged

### Requirement: Reject modification of protected events
The system SHALL reject `update_assignment` for a protected event before issuing any CalDAV write.

#### Scenario: Update rejected for protected event
- **WHEN** `update_assignment` is called for an event that is protected
- **THEN** the operation is rejected before any CalDAV PUT request
- **AND** a German error message explains the event is fixed and cannot be changed

#### Scenario: Update allowed for non-protected event
- **WHEN** `update_assignment` is called for an event that is not protected
- **THEN** the CalDAV PUT proceeds as normal

### Requirement: Reject deletion of protected events
The system SHALL reject `delete_assignment` for a protected event before issuing any CalDAV write.

#### Scenario: Delete rejected for protected event
- **WHEN** `delete_assignment` is called for an event that is protected
- **THEN** the operation is rejected before any CalDAV DELETE request
- **AND** a German error message explains the event is fixed and cannot be removed

#### Scenario: Delete allowed for non-protected event
- **WHEN** `delete_assignment` is called for an event that is not protected
- **THEN** the CalDAV DELETE proceeds as normal
