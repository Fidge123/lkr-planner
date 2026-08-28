## ADDED Requirements

### Requirement: A category set from the planner protects at once
The category deciding whether an event is protected is the linked Daylite project's, so setting it from the planner locks or releases every event referencing that project.
The system SHALL apply the protection rules to the category a project was just given rather than to the one cached before the write.

#### Scenario: Newly fixed project protects its events
- **WHEN** a project is given the category `"Termin FIX geplant"` from the planner
- **THEN** every event referencing that project is treated as protected from that moment
- **AND** a following update, day change, or delete is refused with the German fixed-appointment message

#### Scenario: Releasing the category releases the events
- **WHEN** a project's category is changed from `"Termin FIX geplant"` to another category
- **THEN** events referencing that project are no longer treated as protected
- **AND** they can be edited, moved to another day, and deleted again

#### Scenario: The save that sets the category is not refused by it
- **WHEN** a save writes an assignment and gives its project the category `"Termin FIX geplant"`
- **THEN** the assignment's calendar write is carried out before the category is written
- **AND** it is not refused by the protection the new category creates
