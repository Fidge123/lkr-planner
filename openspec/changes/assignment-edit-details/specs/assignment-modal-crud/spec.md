## ADDED Requirements

### Requirement: Edit the assignment date
The modal SHALL let the planner change the day an assignment falls on, in create mode and in edit mode.

#### Scenario: Date field is pre-filled with the cell's day
- **WHEN** the modal opens from an employee/day cell
- **THEN** the date field shows that cell's day

#### Scenario: Move an assignment to another day
- **WHEN** the planner picks a different date in edit mode and saves
- **THEN** the assignment is written to the new day on the same employee's calendar
- **AND** the grid shows the card in the new cell after the reload
- **AND** the source day no longer shows the card

#### Scenario: Empty or unparseable date blocks the save
- **WHEN** the date field is empty or does not hold a valid date and the planner saves
- **THEN** the save is refused with a German error message
- **AND** nothing is written to the calendar

#### Scenario: Target day lies outside the displayed week
- **WHEN** the planner picks a date that is not part of the week currently shown
- **THEN** the assignment is still written to that date
- **AND** the grid reloads without the card, because the card belongs to another week

### Requirement: Edit the assignment title
The modal SHALL let the planner give an assignment a title of its own that replaces the Daylite project name on the card, and SHALL keep following the project name for an assignment that has no such title.

#### Scenario: Title field defaults to the project name
- **WHEN** the modal opens for an assignment without a title of its own
- **THEN** the title field shows the resolved Daylite project name

#### Scenario: Title field shows an existing override
- **WHEN** the modal opens for an assignment that carries its own title
- **THEN** the title field shows that title, not the project name

#### Scenario: Save a custom title
- **WHEN** the planner types a title different from the project name and saves
- **THEN** the assignment is stored with that title
- **AND** the card in the grid shows it instead of the project name

#### Scenario: Clearing the title returns to the project name
- **WHEN** the planner empties the title field and saves
- **THEN** the assignment no longer carries a title of its own
- **AND** the card shows the resolved Daylite project name again

#### Scenario: Switching project refreshes an untouched title
- **WHEN** the planner picks a different project while the title field still holds the previous project's name unchanged
- **THEN** the title field follows the newly selected project's name

#### Scenario: Switching project keeps an edited title
- **WHEN** the planner has typed a title and then picks a different project
- **THEN** the typed title is kept

### Requirement: Edit the assignment note
The modal SHALL let the planner attach free-text notes to an assignment, stored on the calendar event so other calendar clients show them.

#### Scenario: Note field is pre-filled
- **WHEN** the modal opens for an assignment that carries a note
- **THEN** the note field shows that text, including its line breaks

#### Scenario: Save a note
- **WHEN** the planner writes a note and saves
- **THEN** the note is stored on the event without disturbing its Daylite project reference
- **AND** reopening the modal shows the note unchanged

#### Scenario: Clear a note
- **WHEN** the planner empties the note field and saves
- **THEN** the event keeps its Daylite project reference and carries no note

#### Scenario: Note keeps the assignment classified as an assignment
- **WHEN** an assignment with a note is loaded from the calendar
- **THEN** it is still shown as an assignment card with its project's category color, not as a bare event

## MODIFIED Requirements

### Requirement: Unsaved changes handling
The system SHALL handle unsaved changes properly.

#### Scenario: Close with unsaved changes
- **WHEN** user tries to close modal with unsaved changes
- **THEN** confirmation dialog appears
- **AND** user can save, discard, or cancel

#### Scenario: Editing any field marks the modal dirty
- **WHEN** the planner changes the project, the date, the title, or the note, or adds or removes an attachment
- **THEN** closing the modal shows the unsaved-changes confirmation

#### Scenario: Reverting a field by hand still counts as changed
- **WHEN** the planner edits a field and types the original value back
- **THEN** the modal may still treat the change as unsaved
- **AND** the confirmation dialog is shown on close

### Requirement: Edit existing assignment
The system SHALL allow editing an existing assignment.

#### Scenario: Change project
- **WHEN** user changes the project in edit mode and saves
- **THEN** assignment is updated
- **AND** grid reflects change immediately

#### Scenario: Save leaves untouched fields alone
- **WHEN** the planner changes one field and saves
- **THEN** the assignment's other fields -- project, date, title, note, attachments -- are written back unchanged
- **AND** its position among the day's assignments is unchanged

#### Scenario: Save fails
- **WHEN** writing the assignment to the calendar fails
- **THEN** a German error message is shown in the modal
- **AND** the modal stays open with the planner's input intact

### Requirement: Create new assignment
The system SHALL allow assigning a project to an employee/day.

#### Scenario: Save new assignment
- **WHEN** user selects a project and clicks save
- **THEN** assignment is persisted
- **AND** modal closes
- **AND** weekly grid shows new assignment immediately

#### Scenario: Create with title, note, and attachments
- **WHEN** the planner fills in title, note, or attachments before saving a new assignment
- **THEN** the created event carries them from the start
