## MODIFIED Requirements

### Requirement: Drop targets for rescheduling and reassignment
The system SHALL accept a dragged assignment card on any day cell of any employee and persist the resulting move with the standard assignment time window, matching how the edit modal writes events.
An assignment pinned to times set by hand keeps those times through the drag instead, because a drag changes which day and employee an assignment belongs to, not when it happens.

#### Scenario: Drop on another day of the same employee
- **WHEN** a dragged assignment is dropped on a different day cell of the same employee
- **THEN** the assignment is rescheduled to the target date on the same calendar
- **AND** it is written with the standard assignment time window
- **AND** the grid reloads to show the card in the target cell

#### Scenario: Drop on a different employee
- **WHEN** a dragged assignment is dropped on a cell belonging to a different employee
- **THEN** the assignment is moved to the target employee's calendar on the target date
- **AND** it is written with the standard assignment time window
- **AND** the grid reloads to show the card under the target employee

#### Scenario: A pinned assignment keeps its times
- **WHEN** a dragged assignment carrying pinned times is dropped on another day or another employee
- **THEN** it is written at the target with the same start and end time it had
- **AND** it stays pinned, so the target day's allocation leaves it alone

#### Scenario: Dropping a pinned assignment does not disturb the target day
- **WHEN** a pinned assignment is dropped into a cell that already holds assignments
- **THEN** the assignments already there keep sharing the full window between them
- **AND** the dropped card may overlap them

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
- **AND** the assignment stays in its original place
