## Purpose

Let planners jump the planning view straight to any calendar week from a month-based week picker opened on the grid's calendar week cell, instead of stepping there one week at a time.

## Requirements

### Requirement: The calendar week cell opens the week picker
The system SHALL render the calendar week cell in the planning grid's header corner as a control that opens a week picker anchored to it.
The cell SHALL keep showing the calendar week of the displayed week, and SHALL be labelled for assistive technology as opening the week selection.

#### Scenario: Opening the picker
- **GIVEN** the planning grid shows calendar week 42
- **WHEN** the user clicks the cell labelled "KW 42"
- **THEN** the week picker opens next to the cell

#### Scenario: Incoming week during a swipe
- **GIVEN** the user is pulling a neighbouring week in with a swipe
- **WHEN** the incoming week is drawn over the grid
- **THEN** its calendar week cell shows that week's number
- **AND** it does not offer the week picker

### Requirement: The picker shows two months as week rows
The system SHALL show two consecutive months side by side, each headed by its German month name and year.
Each month SHALL be laid out as rows of weeks from Monday to Sunday, with the ISO calendar week number in front of every row.
Days of the neighbouring months that fill a row SHALL be shown visually subdued.

#### Scenario: Month layout
- **GIVEN** the picker shows October 2026
- **WHEN** the user looks at that month
- **THEN** it is headed "Oktober 2026"
- **AND** its first row starts on Monday 28 September, shown subdued, and is numbered 40
- **AND** the weekday columns are labelled "Mo" to "So"

### Requirement: The picker opens on the displayed week's month and the month before it
The system SHALL open the picker with the month containing the displayed week's Thursday on the right and the month before it on the left.
On the current week this SHALL be the current and the last month.
Reopening the picker SHALL start from this default again, regardless of where it was navigated before.

#### Scenario: Opening on the current week
- **GIVEN** today is 17 October 2026 and the planning view shows the current week
- **WHEN** the user opens the picker
- **THEN** it shows September 2026 and October 2026

#### Scenario: Week spanning two months
- **GIVEN** the displayed week runs from Monday 28 September to Sunday 4 October 2026
- **WHEN** the user opens the picker
- **THEN** it shows September 2026 and October 2026

#### Scenario: Reopening after navigating
- **GIVEN** the user navigated the picker to other months and closed it without choosing a week
- **WHEN** the user opens the picker again
- **THEN** it shows the displayed week's month and the month before it

### Requirement: The picker navigates by month
The system SHALL offer a control to move both shown months one month back and one to move them one month forward, crossing year boundaries.
Navigating SHALL NOT change the week shown in the planning view.

#### Scenario: Moving forward
- **GIVEN** the picker shows September 2026 and October 2026
- **WHEN** the user clicks the next month control
- **THEN** it shows October 2026 and November 2026
- **AND** the planning view still shows the same week

#### Scenario: Crossing the year boundary
- **GIVEN** the picker shows November 2026 and December 2026
- **WHEN** the user clicks the next month control
- **THEN** it shows December 2026 and January 2027

### Requirement: Choosing a day or week number jumps to that week
The system SHALL change the planning view to the calendar week containing the chosen day, or to the week whose number was chosen, and SHALL then close the picker.
The jump SHALL honour the weekend visibility setting, showing the chosen week from Monday to Friday or from Monday to Sunday.
A jump SHALL load its week exactly like any other week change.

#### Scenario: Choosing a day
- **GIVEN** the picker is open
- **WHEN** the user clicks Wednesday 9 September 2026
- **THEN** the planning view shows calendar week 37, starting Monday 7 September 2026
- **AND** the picker is closed

#### Scenario: Choosing a week number
- **GIVEN** the picker is open
- **WHEN** the user clicks the week number 37
- **THEN** the planning view shows calendar week 37

#### Scenario: Choosing a weekend day with the weekend hidden
- **GIVEN** the weekend is hidden
- **WHEN** the user chooses Sunday 13 September 2026
- **THEN** the planning view shows Monday 7 to Friday 11 September 2026

#### Scenario: Choosing a day shown in a neighbouring month's row
- **GIVEN** the picker shows October 2026 with Monday 28 September in its first row
- **WHEN** the user clicks 28 September
- **THEN** the planning view shows calendar week 40

### Requirement: The picker marks the displayed week and today
The system SHALL highlight the row of the displayed week in each shown month that contains it, and SHALL mark today's day distinctly from other days.

#### Scenario: Displayed week and today
- **GIVEN** the planning view shows calendar week 37 and today is 17 October 2026
- **WHEN** the user opens the picker
- **THEN** the row of week 37 is highlighted
- **AND** 17 October is marked as today when October is shown

### Requirement: The picker closes without a jump
The system SHALL close the picker without changing the displayed week when the user presses Escape or clicks outside the picker.

#### Scenario: Escape
- **GIVEN** the picker is open
- **WHEN** the user presses Escape
- **THEN** the picker closes
- **AND** the planning view shows the same week as before

#### Scenario: Click outside
- **GIVEN** the picker is open
- **WHEN** the user clicks the planning grid outside the picker
- **THEN** the picker closes
- **AND** the planning view shows the same week as before
