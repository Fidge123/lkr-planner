## Purpose

Give same-day assignments non-overlapping calendar times instead of all sharing the fixed 08:00-16:00 window, by deterministically splitting the window across the day's assignments and re-allocating it on every create, update, and delete.

## Requirements

### Requirement: Deterministic slot allocation
The system SHALL allocate time slots for same-day assignments in a deterministic manner.

#### Scenario: Single assignment slot
- **GIVEN** 1 assignment for a day
- **WHEN** allocating time slots
- **THEN** the assignment receives the full 08:00-16:00 window

#### Scenario: Two equal assignments
- **GIVEN** 2 assignments for a day
- **WHEN** allocating time slots
- **THEN** assignment 1 receives 08:00-12:00
- **AND** assignment 2 receives 12:00-16:00

#### Scenario: Three equal assignments
- **GIVEN** 3 assignments for a day
- **WHEN** allocating time slots
- **THEN** assignment 1 receives 08:00-10:40
- **AND** assignment 2 receives 10:40-13:20
- **AND** assignment 3 receives 13:20-16:00

#### Scenario: Reordered input produces same output
- **GIVEN** assignments A, B in order [A, B]
- **AND** same assignments A, B in order [B, A]
- **WHEN** allocating time slots for both orderings
- **THEN** both produce identical slot assignments

### Requirement: Non-overlapping slots
The system SHALL ensure allocated slots do not overlap each other.
This covers the assignments that take part in an allocation.
Events excluded from the allocation keep their existing times and are not reserved a share of the window, so they may overlap allocated slots.

#### Scenario: No slot overlap
- **GIVEN** n assignments for a day
- **WHEN** allocating time slots
- **THEN** each slot's end time equals next slot's start time
- **AND** no time is double-allocated

### Requirement: Fixed time window
The system SHALL use fixed 08:00-16:00 window.

#### Scenario: Window boundaries
- **GIVEN** any number of assignments
- **WHEN** allocating time slots
- **THEN** first slot starts at 08:00
- **AND** last slot ends at 16:00

### Requirement: Re-allocate slots on assignment write
The system SHALL re-allocate same-day slots whenever an assignment is created, updated, or deleted, and persist the resulting DTSTART/DTEND to CalDAV.

#### Scenario: Create redistributes the day
- **GIVEN** an employee has 1 assignment on a day occupying 08:00-16:00
- **WHEN** a second assignment is created for the same employee and day
- **THEN** both assignments are re-allocated to 08:00-12:00 and 12:00-16:00
- **AND** the updated times are persisted to CalDAV for both events

#### Scenario: Delete redistributes the day
- **GIVEN** an employee has 3 assignments on a day in thirds of the window
- **WHEN** one of those assignments is deleted
- **THEN** the 2 remaining assignments are re-allocated to 08:00-12:00 and 12:00-16:00
- **AND** the updated times are persisted to CalDAV

#### Scenario: Update that moves an assignment to another day
- **WHEN** an assignment's day is changed
- **THEN** the source day's remaining assignments are re-allocated
- **AND** the target day's assignments (including the moved one) are re-allocated

#### Scenario: Only lkr-planner assignments are re-slotted
- **GIVEN** a day contains lkr-planner assignments alongside bare, absence, or holiday events
- **WHEN** slots are re-allocated
- **THEN** only lkr-planner assignment events have their times changed
- **AND** bare, absence, and holiday events are left untouched

### Requirement: Exclude assignments that cannot be rewritten safely
The system SHALL leave an assignment untouched when rewriting its times could produce invalid iCal or destroy information the assignment carries.
This applies to an event whose end is expressed via DURATION, a resource holding more than one VEVENT, a folded DTSTART or DTEND, an event that belongs to a repeating series, an event that does not start and end on the same day, and an event without a CalDAV resource URL.
Such an event takes no share of the window, so the remaining assignments are still spread across the full window and may overlap it.

#### Scenario: Excluded assignment keeps its times
- **GIVEN** a day contains an assignment whose end is expressed via DURATION
- **WHEN** slots are re-allocated
- **THEN** that assignment's times are not changed
- **AND** no invalid iCal is written for it

#### Scenario: Repeating assignment is never re-slotted
- **GIVEN** a day contains an assignment that repeats through RRULE, RDATE, or RECURRENCE-ID
- **WHEN** slots are re-allocated
- **THEN** that assignment's times are not changed
- **AND** the other occurrences of the series are unaffected

#### Scenario: Multi-day assignment keeps its span
- **GIVEN** a day contains an assignment whose end falls on a later day than its start
- **WHEN** slots are re-allocated
- **THEN** that assignment's times are not changed
- **AND** its span is not collapsed onto the allocated day

#### Scenario: Remaining assignments still use the full window
- **GIVEN** a day contains one excluded assignment and two ordinary assignments
- **WHEN** slots are re-allocated
- **THEN** the two ordinary assignments receive 08:00-12:00 and 12:00-16:00
- **AND** they may overlap the excluded assignment's existing times
