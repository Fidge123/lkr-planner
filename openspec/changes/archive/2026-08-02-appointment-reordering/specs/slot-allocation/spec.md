## MODIFIED Requirements

### Requirement: Deterministic slot allocation
The system SHALL allocate time slots for same-day assignments deterministically, ordered by the planner-controlled order index (ascending) rather than by canonical UID.

#### Scenario: Single assignment slot
- **GIVEN** 1 assignment for a day
- **WHEN** allocating time slots
- **THEN** the assignment receives the full 08:00-16:00 window

#### Scenario: Two assignments ordered by index
- **GIVEN** 2 assignments for a day with order indices 0 and 1
- **WHEN** allocating time slots
- **THEN** the order-index-0 assignment receives 08:00-12:00
- **AND** the order-index-1 assignment receives 12:00-16:00

#### Scenario: Three assignments ordered by index
- **GIVEN** 3 assignments for a day with order indices 0, 1, 2
- **WHEN** allocating time slots
- **THEN** the order-index-0 assignment receives 08:00-10:40
- **AND** the order-index-1 assignment receives 10:40-13:20
- **AND** the order-index-2 assignment receives 13:20-16:00

#### Scenario: Order index determines slot order
- **GIVEN** assignments whose order indices are reassigned so a different assignment is first
- **WHEN** allocating time slots
- **THEN** the assignment with the lowest order index receives the earliest slot
- **AND** changing the order index changes which slot an assignment receives

#### Scenario: Allocation is deterministic for a given index ordering
- **GIVEN** the same assignments with the same order indices presented in any input sequence
- **WHEN** allocating time slots
- **THEN** the resulting slot assignments are identical

#### Scenario: Equal order indices fall back to a stable tie-break
- **GIVEN** 2 assignments for a day that carry the same order index
- **WHEN** allocating time slots
- **THEN** the one that starts earlier is ordered first
- **AND** an identical start time is decided in favor of the longer assignment
- **AND** an identical start time and duration is decided by canonical UID
- **AND** the allocation is still deterministic across runs

#### Scenario: An assignment excluded from re-slotting takes no slot
- **GIVEN** a day contains an assignment `slot-allocation` cannot rewrite
- **WHEN** that assignment is reordered
- **THEN** no slot is allocated for it
- **AND** the assignments that can be rewritten keep sharing the full window between them
