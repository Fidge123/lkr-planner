## ADDED Requirements

### Requirement: Write a day with times set by hand
The system SHALL be able to write an assignment with start and end times a planner entered, instead of the times the even split would give it, and SHALL leave the rest of that day's times alone on such a write.

#### Scenario: Requested times are written as given
- **GIVEN** a day holds three assignments at 08:00-10:40, 10:40-13:20, and 13:20-16:00
- **WHEN** the middle one is written with the requested times 11:00-14:00
- **THEN** it is written as 11:00-14:00

#### Scenario: The day is not re-split around the requested times
- **WHEN** an assignment is written with requested times
- **THEN** the day's other assignments keep the times they already had
- **AND** no even split is applied to that day on that write

#### Scenario: Requested times are not bound by the window
- **WHEN** the requested times start before 08:00 or end after 16:00
- **THEN** they are written as given
- **AND** the fixed window keeps governing every allocation that does apply the even split

#### Scenario: A write without requested times allocates as before
- **WHEN** an assignment is written without requested times
- **THEN** the day is re-allocated by the even split exactly as it is today

#### Scenario: A new assignment can be created with requested times
- **WHEN** an assignment is created with requested times
- **THEN** it is written with those times
- **AND** the day's existing assignments keep the times they already had

### Requirement: Times set by hand last until the day is next re-allocated
Times a planner entered SHALL hold only until the next write that re-allocates their day.
Any create, delete, reorder, or drag affecting that day restores the even split for every assignment on it, and a drag of the assignment itself writes the standard window at its destination.
The system SHALL NOT record that an assignment's times were set by hand.

#### Scenario: Another assignment is created on the day
- **GIVEN** a day holds an assignment written with the times 11:00-14:00
- **WHEN** another assignment is created for that day
- **THEN** the day is re-allocated by the even split
- **AND** the assignment that had 11:00-14:00 receives its share of the window like any other

#### Scenario: Another assignment on the day is deleted
- **GIVEN** a day holds an assignment written with times set by hand alongside two others
- **WHEN** one of the others is deleted
- **THEN** the day is re-allocated by the even split
- **AND** the times set by hand are gone

#### Scenario: The assignment is dragged elsewhere
- **WHEN** an assignment written with times set by hand is dragged to another day or another employee
- **THEN** it is written at the target with the standard assignment time window
- **AND** it takes part in the target day's allocation like any other assignment

#### Scenario: The assignment is reordered within its day
- **WHEN** an assignment written with times set by hand is reordered within its day
- **THEN** the day is re-allocated by the even split
- **AND** the assignment receives the slot its new position gives it

#### Scenario: Saving the assignment again without touching its times
- **WHEN** an assignment written with times set by hand is saved again with only its project, title, or note changed
- **THEN** the day is re-allocated by the even split
- **AND** the times set by hand are gone

#### Scenario: No marker is written
- **WHEN** an assignment is written with times set by hand
- **THEN** the event carries nothing that distinguishes it from an assignment holding allocated times
- **AND** an assignment written before this change is treated no differently

### Requirement: Adjust adjacent assignments to requested times
The system SHALL be able to fit a day's neighbouring assignments to times a planner has just entered, so the day is left without gaps or overlaps between them, and SHALL be able to write the times without touching the neighbours.
The neighbours' fitted times last exactly as long as the requested times do.

#### Scenario: Neighbours are fitted to the requested times
- **GIVEN** a day holds three assignments at 08:00-10:40, 10:40-13:20, and 13:20-16:00
- **WHEN** the middle one is written with the requested times 11:00-14:00 and adjacent adjustment requested
- **THEN** the first assignment is written as 08:00-11:00
- **AND** the third assignment is written as 14:00-16:00

#### Scenario: Without adjustment requested
- **WHEN** the middle assignment of the same day is written with 11:00-14:00 and no adjacent adjustment
- **THEN** only that assignment's times change
- **AND** the neighbours keep their times, leaving a gap before it and an overlap after it

#### Scenario: First and last assignment of the day
- **WHEN** the day's first assignment is written with adjacent adjustment requested
- **THEN** only the assignment after it is fitted, because it has no predecessor
- **AND** the same holds in reverse for the day's last assignment

#### Scenario: Only assignment of the day
- **WHEN** the only assignment of a day is written with requested times and adjacent adjustment
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

#### Scenario: Fitted neighbours are lost with the requested times
- **GIVEN** a day was written with requested times and fitted neighbours
- **WHEN** anything triggers a re-allocation of that day
- **THEN** every assignment on it returns to its share of the even split

## MODIFIED Requirements

### Requirement: Re-allocate slots on assignment write
The system SHALL re-allocate same-day slots whenever an assignment is created, updated, or deleted, and persist the resulting DTSTART/DTEND to CalDAV.
A write that carries times a planner entered is the exception: it writes those times and leaves the day's other assignments alone, as described in "Write a day with times set by hand".

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
Times cannot be entered by hand for such an assignment either, because they could not be written to it.

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
