# Spec Delta

## MODIFIED Requirements

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
- **WHEN** an event's DESCRIPTION contains no `daylite:/<path>` reference
- **AND** the event carries no custom appointment marker (bare event)
- **THEN** the event is protected

#### Scenario: Custom appointment
- **WHEN** an event carries the custom appointment marker and no `daylite:/<path>` reference
- **THEN** the event is not treated as protected
- **AND** no Daylite lookup is performed for it

#### Scenario: Project lookup fails
- **WHEN** the Daylite project referenced by an event cannot be resolved (network error, project not found)
- **THEN** the event is treated as not protected
- **AND** a warning message is shown to the user
- **AND** the event has a warning icon
- **AND** the lookup failure is logged
