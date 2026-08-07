## ADDED Requirements

### Requirement: Assignment title override
An assignment event SHALL be able to carry a title of its own that takes precedence over the resolved Daylite project name wherever the assignment is displayed, and an assignment without one SHALL keep showing the project name.

#### Scenario: Override wins over the project name
- **WHEN** an assignment event carries a title of its own
- **THEN** the card shows that title
- **AND** the project reference, category color, and project status are resolved as before

#### Scenario: No override follows the project
- **WHEN** an assignment event carries no title of its own
- **AND** its Daylite project has been renamed since the event was written
- **THEN** the card shows the project's current name

#### Scenario: Override survives a failed project resolution
- **WHEN** an assignment carries a title of its own and its Daylite project cannot be resolved
- **THEN** the card shows that title inside the German placeholder note instead of the stored project name

#### Scenario: Other calendar clients see the title
- **WHEN** an assignment is read by another calendar client
- **THEN** the event summary is the title of its own when it has one
- **AND** the Daylite project name when it has none

#### Scenario: The replaced title is recorded
- **WHEN** a title of its own is written over the project name an assignment showed until then
- **THEN** the event records that replaced title alongside the new one
- **AND** the recorded title is left as it is by later edits that do not change the title

#### Scenario: Removing the override restores live resolution
- **WHEN** an assignment's title of its own is removed
- **THEN** the event no longer records a replaced title
- **AND** the card shows the Daylite project's current name, not the name recorded when the override was set

### Requirement: Assignment note
An assignment event SHALL be able to carry free-text notes alongside its Daylite project reference, and the reference SHALL stay machine-readable.

#### Scenario: Note and project reference coexist
- **WHEN** an assignment with a note is written and read back
- **THEN** the event is still classified as an assignment carrying its Daylite project reference
- **AND** the note is returned unchanged, including its line breaks

#### Scenario: Note with special characters
- **WHEN** a note contains commas, semicolons, backslashes, or line breaks
- **THEN** it round-trips through the calendar unchanged
- **AND** the written event stays parseable

#### Scenario: Note written outside lkr-planner
- **WHEN** an event's description holds a Daylite reference on its first line and text written by another calendar client below it
- **THEN** that text is shown as the assignment's note
- **AND** it is preserved when the assignment is saved without touching the note

### Requirement: Assignment details survive every write path
Every operation that rewrites an assignment event SHALL preserve its title override, its note, and its attachments unless the operation is explicitly changing them.

#### Scenario: Reschedule by drag
- **WHEN** an assignment carrying a title override, a note, and attachments is dragged to another day of the same employee
- **THEN** the rewritten event still carries all of them

#### Scenario: Move to another employee
- **WHEN** such an assignment is dropped on another employee's cell
- **THEN** the event created on the target calendar carries all of them
- **AND** the source event is deleted as before

#### Scenario: Reorder within a cell
- **WHEN** such an assignment is reordered within its day
- **THEN** the rewritten event still carries all of them

#### Scenario: Slot re-allocation
- **WHEN** the day's assignments are re-slotted after a neighbouring write
- **THEN** the re-slotted events still carry all of them
- **AND** properties the planner never sees, written by other calendar clients, are preserved as before

## MODIFIED Requirements

### Requirement: Move assignment between calendars
The system SHALL move an assignment from one employee's CalDAV calendar to another employee's CalDAV calendar in a single operation and report whether the move completed fully.

#### Scenario: Move to another employee's calendar
- **WHEN** a move is requested with the source assignment href and a target employee reference and date
- **THEN** a new VEVENT carrying the same project reference, project name, title override, note, and attachments is created on the target employee's primary calendar at the target date with the standard assignment time window
- **AND** the original VEVENT is deleted from the source calendar
- **AND** a result indicating a full move with the new CalDAV href is returned

#### Scenario: Source delete fails after target create
- **WHEN** the target VEVENT is created but deleting the source VEVENT fails
- **THEN** the source VEVENT is left in place
- **AND** a result is returned indicating a partial move, carrying both the new href and the source href
- **AND** the operation does not report a plain success

#### Scenario: Target employee has no primary calendar
- **WHEN** a move targets an employee without a configured primary calendar
- **THEN** the operation fails with a German error message
- **AND** the source assignment is left untouched

#### Scenario: Refuse moves into an absence calendar
- **WHEN** a move would write into a configured absence calendar
- **THEN** the operation is refused with a German error message
- **AND** the source assignment is left untouched

#### Scenario: Target create fails
- **WHEN** creating the VEVENT on the target calendar fails
- **THEN** the source VEVENT is not deleted
- **AND** a German error message is returned
