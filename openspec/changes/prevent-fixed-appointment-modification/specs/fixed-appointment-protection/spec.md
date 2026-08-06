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
The system SHALL reject `update_assignment` for a protected event before issuing any CalDAV write, unless the caller explicitly overrides the protection.

#### Scenario: Update rejected for protected event
- **WHEN** `update_assignment` is called for an event that is protected
- **THEN** the operation is rejected before any CalDAV PUT request
- **AND** a German error message explains the event is fixed and cannot be changed

#### Scenario: Update allowed for non-protected event
- **WHEN** `update_assignment` is called for an event that is not protected
- **THEN** the CalDAV PUT proceeds as normal

#### Scenario: Update allowed with an explicit override
- **WHEN** `update_assignment` is called for a protected event with the protection override set
- **THEN** no protection check is performed
- **AND** the CalDAV PUT proceeds as normal

### Requirement: Reject deletion of protected events
The system SHALL reject `delete_assignment` for a protected event before issuing any CalDAV write, unless the caller explicitly overrides the protection.

#### Scenario: Delete rejected for protected event
- **WHEN** `delete_assignment` is called for an event that is protected
- **THEN** the operation is rejected before any CalDAV DELETE request
- **AND** a German error message explains the event is fixed and cannot be removed

#### Scenario: Delete allowed for non-protected event
- **WHEN** `delete_assignment` is called for an event that is not protected
- **THEN** the CalDAV DELETE proceeds as normal

#### Scenario: Delete allowed with an explicit override
- **WHEN** `delete_assignment` is called for a protected event with the protection override set
- **THEN** no protection check is performed
- **AND** the CalDAV DELETE proceeds as normal

### Requirement: Allow replanning a protected event within its day
The day is the committed part of a fixed appointment, so the system SHALL allow a protected event to be reassigned to another employee or reordered as long as it keeps its date.

#### Scenario: Move to another employee on the same day
- **WHEN** `move_assignment` is called for a protected event
- **AND** the target date equals the event's current date
- **THEN** the move proceeds as normal

#### Scenario: Move to another day
- **WHEN** `move_assignment` is called for a protected event
- **AND** the target date differs from the event's current date
- **THEN** the operation is rejected before any CalDAV write
- **AND** a German error message explains the event is fixed and cannot be moved to another day

#### Scenario: Reorder within the day
- **WHEN** `reorder_assignment` is called for a protected event
- **THEN** the reorder proceeds as normal

#### Scenario: Re-slotting a day rewrites a protected event's times
- **WHEN** an assignment is created, updated or deleted on a day that also holds a protected event
- **THEN** the protected event's DTSTART and DTEND are re-slotted as normal
