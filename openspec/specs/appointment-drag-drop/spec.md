## Purpose

Let planners reschedule and reassign an existing assignment directly in the grid by dragging its card to another day or another employee, instead of reopening the edit modal for every move.

## Requirements

### Requirement: Draggable assignment cards
The system SHALL allow assignment cards (`kind: "assignment"`) in the planning grid to be picked up and dragged by their body, leaving the card's action controls free to be pressed.

#### Scenario: Assignment card is draggable
- **WHEN** the user presses and drags an assignment card's body past the activation threshold
- **THEN** a drag operation starts carrying that assignment's identity (UID, href, source employee, source date)
- **AND** the source card is visually marked as being dragged

#### Scenario: Bare and absence events are not draggable
- **WHEN** the user attempts to drag a bare or absence event card
- **THEN** no drag operation starts
- **AND** the card remains in place

#### Scenario: Assignments with an unresolved project are not draggable
- **WHEN** an assignment's Daylite project could not be resolved and its card shows the German placeholder text
- **THEN** the card is not draggable until the project data is available
- **AND** its edit action still opens the edit modal

#### Scenario: Pressing the card without passing the drag threshold
- **WHEN** the user presses an assignment card's body without moving past the activation threshold
- **THEN** nothing happens, because the card body is no longer a control
- **AND** the edit modal is reached through the card's edit action instead

#### Scenario: Pressing a card action never starts a drag
- **WHEN** the user presses and moves the pointer on one of the card's action controls
- **THEN** no drag operation starts and the card stays in place

### Requirement: Drop targets for rescheduling and reassignment
The system SHALL accept a dragged assignment card on any day cell of any employee and persist the resulting move with the standard assignment time window, matching how the edit modal writes events, unless the drop would move a protected event to another day (see `fixed-appointment-protection`).
The card SHALL be shown in its target cell as soon as it is dropped, and only the rows of the source and target employees SHALL be refreshed once the write completes.

#### Scenario: Drop on another day of the same employee
- **WHEN** a dragged assignment is dropped on a different day cell of the same employee
- **THEN** the card is shown in the target cell immediately, without times until the day is re-slotted
- **AND** the assignment is rescheduled to the target date on the same calendar
- **AND** it is written with the standard assignment time window
- **AND** the employee's row is refreshed without the loading spinner once the write completes

#### Scenario: Drop on a different employee
- **WHEN** a dragged assignment is dropped on a cell belonging to a different employee
- **THEN** the card is shown under the target employee immediately
- **AND** the assignment is moved to the target employee's calendar on the target date
- **AND** it is written with the standard assignment time window
- **AND** the rows of both employees are refreshed without the loading spinner once the write completes

#### Scenario: Drop a protected assignment on another employee on the same day
- **WHEN** a dragged assignment whose linked Daylite project has category `"Termin FIX geplant"` is dropped on a cell of a different employee on its own date
- **THEN** the assignment is moved to the target employee's calendar
- **AND** the card is shown under the target employee

#### Scenario: Drop a protected assignment on another day
- **WHEN** a dragged assignment whose linked Daylite project has category `"Termin FIX geplant"` is dropped on a cell of a different date
- **THEN** the move is rejected before any CalDAV write
- **AND** the German error message returned by the backend is shown
- **AND** the card stays in its original cell throughout, without being shown in the target cell

#### Scenario: Drop lands on the day cell without within-cell positioning
- **WHEN** a dragged assignment is dropped onto a cell that already contains other cards
- **THEN** the assignment is placed in that cell
- **AND** its position relative to existing cards is not controlled by the drop location

#### Scenario: Drop on the originating cell
- **WHEN** a dragged assignment is dropped on the same employee and date it came from
- **THEN** no persistence call is made
- **AND** the grid is unchanged

#### Scenario: Drop on an employee without a configured calendar
- **WHEN** a dragged assignment is dropped on a cell of an employee that has no primary calendar
- **THEN** the move is rejected
- **AND** a German error message is shown
- **AND** the refreshed rows show the assignment in its original place

#### Scenario: Drop into a week other than the source week
- **WHEN** a dragged assignment is dropped after edge-hover navigation into another week
- **THEN** the card is shown in the target cell of the displayed week
- **AND** the cached copy of the source week is discarded so it is fetched again when shown

### Requirement: Drag-and-drop visual affordances
The system SHALL provide visual feedback that communicates valid drop targets during a drag.

#### Scenario: Render a drag preview that survives week navigation
- **WHEN** an assignment is being dragged
- **THEN** a drag preview follows the pointer independently of the source card's DOM node

#### Scenario: Highlight the hovered drop target
- **WHEN** a dragged assignment is over a droppable day cell
- **THEN** that cell shows a drop-target indicator

#### Scenario: Clear indicators when the drag ends
- **WHEN** the drag operation ends by drop or cancel
- **THEN** all drag and drop-target indicators are removed

### Requirement: Edge-hover week navigation during drag
The system SHALL navigate to the adjacent week when a dragged assignment dwells over the left or right edge of the grid.

#### Scenario: Hover at the right edge advances the week
- **WHEN** a dragged assignment is held over the right edge zone of the grid for the dwell duration
- **THEN** the grid navigates to the next week
- **AND** the drag operation continues so the card can be dropped in the newly shown week

#### Scenario: Hover at the left edge goes to the previous week
- **WHEN** a dragged assignment is held over the left edge zone of the grid for the dwell duration
- **THEN** the grid navigates to the previous week
- **AND** the drag operation continues

#### Scenario: Leaving the edge cancels pending navigation
- **WHEN** the dragged assignment leaves the edge zone before the dwell duration elapses
- **THEN** no week navigation occurs

#### Scenario: Repeated dwell jumps multiple weeks
- **WHEN** the dragged assignment remains in an edge zone after a navigation
- **THEN** the dwell timer restarts and navigation repeats for each completed dwell, allowing several weeks to be crossed in one drag

### Requirement: Reconcile a partially completed cross-employee move
The system SHALL let the user reconcile a cross-employee move that created the assignment on the target calendar but failed to delete it from the source calendar.

#### Scenario: Source delete fails after target create
- **WHEN** a cross-employee move reports that the target copy was created but the source copy could not be deleted
- **THEN** a German reconciliation dialog is shown offering: retry deleting the source, keep the old copy and delete the new one, or keep both
- **AND** the dialog blocks until the user chooses

#### Scenario: Retry deleting the source
- **WHEN** the user chooses to retry deleting the source
- **THEN** the source copy is deleted
- **AND** on success the dialog closes and the grid reloads

#### Scenario: Keep the old copy and delete the new one
- **WHEN** the user chooses to keep the old copy and delete the new one
- **THEN** the newly created target copy is deleted
- **AND** the dialog closes and the grid reloads

#### Scenario: Keep both copies
- **WHEN** the user chooses to keep both copies
- **THEN** the dialog closes leaving both copies in place
- **AND** the grid reloads showing both
