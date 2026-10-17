# Spec Delta

## Purpose

Let planners plan work that has no Daylite project, such as workshop days, trainings or vehicle service, as planner-managed calendar entries with a free-text title that behave like project assignments in the weekly grid.

## ADDED Requirements

### Requirement: Identify custom appointments
The system SHALL identify a CalDAV event as a custom appointment when the first line of its DESCRIPTION is exactly `lkr-planner:custom`.
The first line is read with the same tolerance for leading and trailing whitespace, BOM and zero-width space as the `daylite:` reference.

#### Scenario: Event with the custom marker
- **WHEN** the first DESCRIPTION line of an event in an employee's primary calendar is `lkr-planner:custom`
- **THEN** the event is treated as a custom appointment

#### Scenario: Further description lines are ignored
- **WHEN** the first DESCRIPTION line is `lkr-planner:custom`
- **AND** further lines follow it
- **THEN** the event is still treated as a custom appointment

#### Scenario: Marker with surrounding whitespace or BOM
- **WHEN** the first DESCRIPTION line is `lkr-planner:custom` preceded or followed by whitespace, a BOM or a zero-width space
- **THEN** the event is treated as a custom appointment

#### Scenario: Marker not on the first line
- **WHEN** `lkr-planner:custom` appears in the DESCRIPTION but not as its first line
- **THEN** the event is treated as a bare event

#### Scenario: Other first line stays bare
- **WHEN** the first DESCRIPTION line is neither `lkr-planner:custom` nor a `daylite:/<path>` reference
- **THEN** the event is treated as a bare event

#### Scenario: Marker removed in another calendar client
- **WHEN** another calendar client changes the first DESCRIPTION line of a custom appointment
- **THEN** the event is treated as a bare event from then on

### Requirement: Write custom appointments to CalDAV
The system SHALL write a custom appointment as a VEVENT in the employee's primary CalDAV calendar whose SUMMARY is the title and whose DESCRIPTION is `lkr-planner:custom`.

#### Scenario: Create a custom appointment
- **WHEN** a custom appointment is created
- **THEN** a VEVENT with a stable UUID as its UID is written to the employee's primary calendar
- **AND** its SUMMARY is the entered title
- **AND** its DESCRIPTION is `lkr-planner:custom`

#### Scenario: Title is written verbatim
- **WHEN** the entered title contains commas, semicolons, backslashes or umlauts
- **THEN** the title read back from CalDAV equals the entered title

#### Scenario: Write fails
- **WHEN** the CalDAV write for a custom appointment fails
- **THEN** a German error message is shown to the user
- **AND** the user can retry from the modal

#### Scenario: Absence calendar is never written
- **WHEN** a custom appointment write would target a configured absence calendar
- **THEN** the write is rejected before any network request

### Requirement: Create a custom appointment from the modal
The system SHALL let the user create a custom appointment from the assignment modal by switching the appointment type to "Eigener Termin" and entering a title.

#### Scenario: Type switch defaults to project
- **WHEN** the modal opens in create mode
- **THEN** an appointment type switch with the German options "Projekt" and "Eigener Termin" is shown
- **AND** "Projekt" is selected

#### Scenario: Custom mode replaces the project picker
- **WHEN** the user selects "Eigener Termin"
- **THEN** the project picker and its suggestions are hidden
- **AND** a text field labeled "Titel" is shown and focused

#### Scenario: Blank title cannot be saved
- **WHEN** the title is empty or consists only of whitespace
- **THEN** the save control is disabled

#### Scenario: Save a custom appointment
- **WHEN** the user enters a title and clicks save
- **THEN** a custom appointment is persisted for the preselected employee and day with the trimmed title
- **AND** the modal closes
- **AND** the grid reloads and shows the new card

#### Scenario: Entered title counts as unsaved change
- **WHEN** the user has entered a title and tries to close the modal without saving
- **THEN** the unsaved-changes confirmation appears

### Requirement: Edit and delete a custom appointment from the modal
The system SHALL open the assignment modal in edit mode when the user clicks a custom appointment card and SHALL let the user change its title or delete it.
The appointment type of an existing entry SHALL NOT be changeable.

#### Scenario: Edit mode is prefilled
- **WHEN** the user clicks a custom appointment card
- **THEN** the modal opens in edit mode with the title field holding the appointment's current title
- **AND** no project picker is shown

#### Scenario: No type switch in edit mode
- **WHEN** the modal opens in edit mode for a custom appointment or a project assignment
- **THEN** no appointment type switch is shown

#### Scenario: Change the title
- **WHEN** the user changes the title and saves
- **THEN** the existing VEVENT is updated with the new title under the same UID
- **AND** it keeps `lkr-planner:custom` as its DESCRIPTION
- **AND** the grid reflects the change

#### Scenario: Blank title cannot be saved in edit mode
- **WHEN** the user clears the title of an existing custom appointment
- **THEN** the save control is disabled

#### Scenario: Delete a custom appointment
- **WHEN** the user deletes a custom appointment and confirms
- **THEN** the VEVENT is removed from CalDAV
- **AND** the grid shows the cell without it

### Requirement: Custom appointments are never protected
The system SHALL treat a custom appointment as not protected, so updating, deleting, moving to another day and reordering it proceed without a protection check failing.

#### Scenario: Modal shows no fixed-appointment notice
- **WHEN** the modal opens for a custom appointment
- **THEN** no fixed-appointment notice and no unlock control are shown
- **AND** the save and delete controls are enabled

#### Scenario: Move to another day
- **WHEN** a custom appointment is moved to another day
- **THEN** the move proceeds as normal

### Requirement: Render custom appointment cards
The system SHALL render a custom appointment as an editable, draggable card showing its title and times, without the affordances that depend on a Daylite project.

#### Scenario: Card content
- **WHEN** a custom appointment is rendered
- **THEN** the card shows the title and, for a timed event, its start and end time
- **AND** it uses the same surface color as other cards
- **AND** its strip keeps the default muted color

#### Scenario: No project affordances
- **WHEN** a custom appointment card is rendered
- **THEN** it shows no project status, no unresolved-project warning and no Daylite deep link
- **AND** it offers no highlight toggle

#### Scenario: Never highlighted
- **WHEN** a project highlight is active in the week
- **THEN** no custom appointment card is highlighted

### Requirement: Custom appointments are planned like project assignments
The system SHALL let custom appointments take part in drag-and-drop, intra-day reordering, order indices and slot allocation in the same way as project assignments.

#### Scenario: Shares the day's slots
- **GIVEN** an employee has one project assignment and one custom appointment on a day
- **WHEN** slots are re-allocated
- **THEN** the two receive 08:00-12:00 and 12:00-16:00 in order-index order

#### Scenario: Move to another employee keeps the title
- **WHEN** a custom appointment is dragged to another employee's cell
- **THEN** the new VEVENT on the target calendar carries the same title and `lkr-planner:custom` as its DESCRIPTION
- **AND** the source VEVENT is deleted

#### Scenario: Reorder within a cell
- **WHEN** a custom appointment is dragged above a project assignment in the same cell
- **THEN** its order index is set before the project assignment
- **AND** the new order is persisted
