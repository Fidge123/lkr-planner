## ADDED Requirements

### Requirement: Manually timed assignments are excluded from allocation
The system SHALL let an assignment be pinned to start and end times set by hand, and SHALL exclude such an assignment from slot allocation for as long as the pin holds.
A pinned assignment keeps its times through every re-allocation and takes no share of the window, so the remaining assignments still spread across the full 08:00-16:00 window and may overlap it.
This is the same treatment an assignment that cannot be rewritten safely receives, but chosen by the planner rather than forced by the event's shape.

#### Scenario: Pinned assignment keeps its times
- **GIVEN** a day contains an assignment pinned to 06:00-09:00
- **WHEN** slots are re-allocated for that day
- **THEN** its times are still 06:00-09:00

#### Scenario: Remaining assignments still use the full window
- **GIVEN** a day contains one pinned assignment and two unpinned ones
- **WHEN** slots are re-allocated
- **THEN** the two unpinned assignments receive 08:00-12:00 and 12:00-16:00
- **AND** they may overlap the pinned assignment's times

#### Scenario: Pinned times are not bound by the window
- **WHEN** an assignment is pinned to times that start before 08:00 or end after 16:00
- **THEN** those times are written as given
- **AND** the fixed window keeps governing the unpinned assignments only

#### Scenario: A day of only pinned assignments is not re-allocated
- **GIVEN** every assignment on a day is pinned
- **WHEN** a write triggers re-allocation for that day
- **THEN** no assignment's times are changed

#### Scenario: Releasing the pin returns the assignment to allocation
- **WHEN** an assignment's pin is removed
- **THEN** it takes part in the next allocation for its day
- **AND** it receives its share of the window like any other assignment

#### Scenario: Pinned assignment keeps its position in the day's order
- **GIVEN** a day contains pinned and unpinned assignments
- **WHEN** the day is re-sequenced
- **THEN** the pinned assignment keeps its position among the day's cards
- **AND** its order index is maintained as it is for any other excluded assignment

### Requirement: Adjust adjacent assignments to pinned times
The system SHALL be able to fit a day's neighbouring assignments to times a planner has just set, so the day is left without gaps or overlaps between them, and SHALL be able to write the times without touching the neighbours.

#### Scenario: Neighbours are fitted to the new times
- **GIVEN** a day holds three assignments at 08:00-10:40, 10:40-13:20, and 13:20-16:00
- **WHEN** the middle one is set to 11:00-14:00 with adjacent adjustment requested
- **THEN** the first assignment is written as 08:00-11:00
- **AND** the third assignment is written as 14:00-16:00
- **AND** all three are pinned, because none of their times follow the even split any more

#### Scenario: Adjustment without adjustment requested
- **WHEN** the middle assignment of the same day is set to 11:00-14:00 without adjacent adjustment
- **THEN** only that assignment's times change
- **AND** the neighbours keep their times, leaving a gap before it and an overlap after it

#### Scenario: First and last assignment of the day
- **WHEN** the day's first assignment is adjusted with adjacent adjustment requested
- **THEN** only the assignment after it is fitted, because it has no predecessor
- **AND** the same holds in reverse for the day's last assignment

#### Scenario: Only assignment of the day
- **WHEN** the only assignment of a day is given times with adjacent adjustment requested
- **THEN** no other event is written

#### Scenario: Adjustment would leave a neighbour without duration
- **WHEN** the requested times would push a neighbour's start to or past its end
- **THEN** the write is refused with a German error message naming the conflicting assignment
- **AND** neither the edited assignment nor its neighbours are changed

#### Scenario: Adjustment never touches non-assignments
- **GIVEN** a day contains bare, absence, or holiday events between two assignments
- **WHEN** adjacent adjustment runs
- **THEN** those events are left untouched
- **AND** the neighbours it fits are the adjacent assignments, not the adjacent events

#### Scenario: A neighbour that cannot be rewritten safely is skipped
- **GIVEN** the assignment adjacent to the edited one is excluded from re-slotting because it cannot be rewritten safely
- **WHEN** adjacent adjustment runs
- **THEN** that neighbour is left untouched
- **AND** the edited assignment is still written with its requested times

## MODIFIED Requirements

### Requirement: Re-allocate slots on assignment write
The system SHALL re-allocate same-day slots whenever an assignment is created, updated, or deleted, and persist the resulting DTSTART/DTEND to CalDAV.
Assignments pinned to times set by hand do not take part, as described in "Manually timed assignments are excluded from allocation".

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

#### Scenario: A pinned assignment is created
- **GIVEN** an employee has 1 assignment on a day occupying 08:00-16:00
- **WHEN** a second assignment is created for the same day with times set by hand
- **THEN** the new assignment is written with those times
- **AND** the existing assignment keeps the full 08:00-16:00 window, because only one assignment takes part in the allocation

#### Scenario: A pinned assignment is deleted
- **GIVEN** a day holds two unpinned assignments and one pinned one
- **WHEN** the pinned assignment is deleted
- **THEN** the two remaining assignments are re-allocated to 08:00-12:00 and 12:00-16:00

### Requirement: Exclude assignments that cannot be rewritten safely
The system SHALL leave an assignment untouched when rewriting its times could produce invalid iCal or destroy information the assignment carries.
This applies to an event whose end is expressed via DURATION, a resource holding more than one VEVENT, a folded DTSTART or DTEND, an event that belongs to a repeating series, an event that does not start and end on the same day, and an event without a CalDAV resource URL.
Such an event takes no share of the window, so the remaining assignments are still spread across the full window and may overlap it.
An assignment excluded this way cannot be pinned to times set by hand either, because the planner's times could not be written to it.

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

#### Scenario: Time fields are unavailable for such an assignment
- **WHEN** the modal opens for an assignment that cannot be rewritten safely
- **THEN** its start and end time fields are shown but cannot be edited
- **AND** a German hint explains that this assignment's times cannot be changed
