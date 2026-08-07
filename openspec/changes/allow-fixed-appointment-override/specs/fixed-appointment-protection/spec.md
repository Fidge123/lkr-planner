## ADDED Requirements

### Requirement: Deliberate override of the protection
The system SHALL skip the protection check for `update_assignment` and `delete_assignment` when the caller sets an explicit override, so a user who deliberately unlocked a fixed appointment can change or remove it.

#### Scenario: Update with an override
- **WHEN** `update_assignment` is called for a protected event with the override set
- **THEN** no protection check is performed
- **AND** the CalDAV PUT proceeds as normal

#### Scenario: Delete with an override
- **WHEN** `delete_assignment` is called for a protected event with the override set
- **THEN** no protection check is performed
- **AND** the CalDAV DELETE proceeds as normal

#### Scenario: Override is per write, not per event
- **WHEN** a later `update_assignment` or `delete_assignment` for the same event is called without the override
- **THEN** the protection check runs again
- **AND** the operation is rejected as before

#### Scenario: Move carries no override
- **WHEN** `move_assignment` is called for a protected event with a target date other than the event's own
- **THEN** the operation is rejected regardless of any override the caller supplies
