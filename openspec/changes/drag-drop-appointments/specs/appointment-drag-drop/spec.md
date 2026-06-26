## ADDED Requirements

### Requirement: Draggable assignment cards
The system SHALL allow assignment cards (`kind: "assignment"`) in the planning grid to be picked up and dragged.

#### Scenario: Assignment card is draggable
- **WHEN** the user presses and drags an assignment card
- **THEN** a drag operation starts carrying that assignment's identity (UID, href, source employee, source date)
- **AND** the source card is visually marked as being dragged

#### Scenario: Bare and absence events are not draggable
- **WHEN** the user attempts to drag a bare or absence event card
- **THEN** no drag operation starts
- **AND** the card remains in place

#### Scenario: Click still opens the edit modal
- **WHEN** the user clicks an assignment card without dragging it
- **THEN** the edit modal opens as before

### Requirement: Drop targets for rescheduling and reassignment
The system SHALL accept a dragged assignment card on any day cell of any employee and persist the resulting move.

#### Scenario: Drop on another day of the same employee
- **WHEN** a dragged assignment is dropped on a different day cell of the same employee
- **THEN** the assignment is rescheduled to the target date on the same calendar
- **AND** the grid reloads to show the card in the target cell

#### Scenario: Drop on a different employee
- **WHEN** a dragged assignment is dropped on a cell belonging to a different employee
- **THEN** the assignment is moved to the target employee's calendar on the target date
- **AND** the grid reloads to show the card under the target employee

#### Scenario: Drop on the originating cell
- **WHEN** a dragged assignment is dropped on the same employee and date it came from
- **THEN** no persistence call is made
- **AND** the grid is unchanged

#### Scenario: Drop on an employee without a configured calendar
- **WHEN** a dragged assignment is dropped on a cell of an employee that has no primary calendar
- **THEN** the move is rejected
- **AND** a German error message is shown
- **AND** the assignment stays in its original place

### Requirement: Drag-and-drop visual affordances
The system SHALL provide visual feedback that communicates valid drop targets during a drag.

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
