# Spec Delta

## MODIFIED Requirements

### Requirement: Re-allocate slots on assignment write
The system SHALL re-allocate same-day slots whenever an assignment or a custom appointment is created, updated, or deleted, and persist the resulting DTSTART/DTEND to CalDAV.

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

#### Scenario: Custom appointments are re-slotted with assignments
- **GIVEN** a day contains lkr-planner assignments and custom appointments
- **WHEN** slots are re-allocated
- **THEN** the window is split across the assignments and custom appointments together in order-index order
- **AND** each custom appointment keeps its title and its custom appointment marker
