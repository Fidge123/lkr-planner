## ADDED Requirements

### Requirement: Persisted order index for same-day assignments
The system SHALL maintain a persisted order index that defines the position of each lkr-planner assignment among its same-day, same-employee siblings.

#### Scenario: Cell renders in order-index order
- **WHEN** a day cell contains multiple assignments
- **THEN** the cards are rendered sorted by their order index

#### Scenario: Order index persists across devices
- **WHEN** an assignment's order index is set on one device
- **THEN** another device loading the same week sees the same order

#### Scenario: Dense re-sequencing on membership change
- **WHEN** the set of assignments in a day changes by create, delete, reorder, or move
- **THEN** the affected day's order indices are re-sequenced to a contiguous order
- **AND** the new order is persisted

#### Scenario: Assignment excluded from re-slotting keeps its order position
- **GIVEN** a day contains an assignment that `slot-allocation` excludes from re-slotting
- **WHEN** the day's order indices are re-sequenced
- **THEN** that assignment still receives an order index and renders at that position
- **AND** its times are not rewritten, so they may disagree with its position

### Requirement: Intra-day reorder via drag
The system SHALL let the user drag an assignment within its day cell to change its order index without changing its date or employee.

#### Scenario: Reorder within a cell
- **WHEN** the user drags an assignment above another assignment in the same cell
- **THEN** the dragged assignment's order index is set before the target
- **AND** the cell re-renders in the new order
- **AND** the new order is persisted

#### Scenario: Reorder does not change date or employee
- **WHEN** an assignment is reordered within its cell
- **THEN** its date and employee are unchanged
- **AND** no cross-calendar move occurs

### Requirement: Precise before/after placement on cross-cell drops
The system SHALL let cross-day and cross-employee drops land the dragged assignment at a specific position before or after an existing card in the target cell.

#### Scenario: Drop before a target card
- **WHEN** a dragged assignment is dropped onto the upper part of an existing card in the target cell
- **THEN** the dragged assignment is placed before that card
- **AND** the target cell is re-sequenced and persisted

#### Scenario: Drop after a target card
- **WHEN** a dragged assignment is dropped onto the lower part of an existing card in the target cell
- **THEN** the dragged assignment is placed after that card
- **AND** the target cell is re-sequenced and persisted

#### Scenario: Drop into an empty area of the cell
- **WHEN** a dragged assignment is dropped onto a target cell below all existing cards
- **THEN** the dragged assignment is placed last
- **AND** the target cell is re-sequenced and persisted

### Requirement: Drop preview during a drag
The system SHALL show, while an assignment is being dragged, which day cell would receive it and the exact position it would take inside that cell.

#### Scenario: Target cell stays outlined over its cards
- **WHEN** the pointer is anywhere inside a day cell, including over one of its existing cards
- **THEN** that cell is outlined as the drop target

#### Scenario: Preview marks the landing position
- **WHEN** a drag hovers a position in a cell
- **THEN** a placeholder is rendered at the position the assignment would take
- **AND** the placeholder moves as the pointer moves to another position

#### Scenario: Preview does not disturb the grid
- **WHEN** the placeholder is shown in a cell
- **THEN** the dragged assignment's card is taken out of that cell's layout
- **AND** the position under the pointer does not change as a result of showing the placeholder
