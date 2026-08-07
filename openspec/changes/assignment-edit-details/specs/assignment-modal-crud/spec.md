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

### Requirement: Edit the assignment times
The modal SHALL let the planner set an assignment's start and end time by hand, which pins the assignment to those times, and SHALL let the planner hand it back to automatic slot allocation.

#### Scenario: Time fields show the current slot
- **WHEN** the modal opens for an assignment
- **THEN** the start and end time fields show the times the assignment currently holds, whether allocated or set by hand

#### Scenario: Set times by hand
- **WHEN** the planner changes a time and saves
- **THEN** the assignment is written with the entered times
- **AND** it is pinned, so later writes on that day leave its times alone

#### Scenario: Return an assignment to automatic allocation
- **WHEN** the planner empties both time fields and saves
- **THEN** the assignment is no longer pinned
- **AND** it receives its share of the day's window again

#### Scenario: End before or equal to start blocks the save
- **WHEN** the entered end time is not after the entered start time
- **THEN** the save is refused with a German error message
- **AND** nothing is written to the calendar

#### Scenario: Only one time field filled blocks the save
- **WHEN** exactly one of the two time fields holds a value
- **THEN** the save is refused with a German error message explaining that both times are needed

#### Scenario: Times outside the standard window are accepted
- **WHEN** the planner enters times starting before 08:00 or ending after 16:00
- **THEN** the save is accepted
- **AND** the assignment is written with those times

#### Scenario: Times are not editable for an assignment that cannot be rewritten
- **WHEN** the modal opens for an assignment excluded from re-slotting because it cannot be rewritten safely
- **THEN** the time fields cannot be edited
- **AND** the rest of the modal stays editable

### Requirement: Adjust adjacent assignments
The modal SHALL offer a checkbox, ticked by default, that fits the day's neighbouring assignments to the times just entered.

#### Scenario: Checkbox is offered with the time fields
- **WHEN** the modal is open
- **THEN** a checkbox with a German label offering to adjust the adjacent assignments is shown next to the time fields
- **AND** it is ticked by default

#### Scenario: Save with adjustment
- **WHEN** the planner sets times with the checkbox ticked and saves
- **THEN** the assignment before it in the day ends where it now starts
- **AND** the assignment after it starts where it now ends
- **AND** the grid shows all of them in their new times after the reload

#### Scenario: Save without adjustment
- **WHEN** the planner unticks the checkbox and saves
- **THEN** only the edited assignment's times change

#### Scenario: Adjustment is refused
- **WHEN** the adjustment would leave a neighbouring assignment without duration
- **THEN** a German error message is shown in the modal
- **AND** the modal stays open with the planner's input intact
- **AND** nothing is written to the calendar

#### Scenario: Checkbox has no effect without a time change
- **WHEN** the planner saves with the checkbox ticked but has not changed either time
- **THEN** no neighbouring assignment is written

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
- **WHEN** the planner changes the project, the date, either time, the title, or the note
- **THEN** closing the modal shows the unsaved-changes confirmation

#### Scenario: Toggling the adjustment checkbox alone does not
- **WHEN** the planner only ticks or unticks the adjacent-adjustment checkbox
- **THEN** the modal is not treated as changed, because the checkbox governs how a time change is written rather than being a change itself

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
- **THEN** the assignment's other fields -- project, date, times, title, note -- are written back unchanged
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

#### Scenario: Create with title and note
- **WHEN** the planner fills in a title or a note before saving a new assignment
- **THEN** the created event carries them from the start

#### Scenario: Create with times
- **WHEN** the planner enters times for a new assignment
- **THEN** it is created pinned to those times
- **AND** the day's other assignments are allocated without it

#### Scenario: Create without times
- **WHEN** the planner leaves both time fields empty for a new assignment
- **THEN** it takes part in the day's allocation like any other new assignment
