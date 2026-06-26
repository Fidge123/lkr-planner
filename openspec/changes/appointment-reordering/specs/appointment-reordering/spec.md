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
