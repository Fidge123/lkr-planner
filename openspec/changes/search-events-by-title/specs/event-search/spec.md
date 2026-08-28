## ADDED Requirements

### Requirement: Opening the search
The system SHALL offer a search button in the application header that opens a search modal.
The modal SHALL hold a query field, and SHALL be dismissable without changing the week the grid shows.

#### Scenario: The header offers a search button
- **WHEN** the application is rendered
- **THEN** the header includes a search button carrying a German label

#### Scenario: The button opens the modal
- **WHEN** the user activates the search button
- **THEN** the search modal is shown with an empty query field

#### Scenario: Dismissing leaves the grid alone
- **GIVEN** the search modal is open
- **WHEN** the user dismisses it without choosing a result
- **THEN** the grid shows the same week it showed before

### Requirement: Running a search
The system SHALL search only when the query is submitted, and SHALL NOT search while the query is being typed.
A query that is empty or only whitespace SHALL NOT be searched.

#### Scenario: Typing does not search
- **GIVEN** the search modal is open
- **WHEN** the user types into the query field without submitting
- **THEN** no search is run

#### Scenario: Submitting searches
- **GIVEN** a query has been typed
- **WHEN** the user submits it
- **THEN** a search for that query is run

#### Scenario: A blank query is not searched
- **WHEN** the user submits an empty or whitespace-only query
- **THEN** no search is run and no results are shown

#### Scenario: The search reports that it is running
- **WHEN** a search is running
- **THEN** the modal shows a loading state until the results arrive

### Requirement: Title matching
The system SHALL treat an event as a match when the submitted query, trimmed and lowercased, occurs anywhere in the event's title lowercased.
Lowercasing SHALL be Unicode-aware, so that a query matches a title differing from it only in capitalisation of a non-ASCII letter.

#### Scenario: A fragment matches
- **GIVEN** an event titled `Neubau Müller`
- **WHEN** the user searches for `bau`
- **THEN** that event is a result

#### Scenario: Capitalisation is ignored
- **GIVEN** an event titled `Neubau Müller`
- **WHEN** the user searches for `müller`
- **THEN** that event is a result

#### Scenario: Capitalisation of a non-ASCII letter is ignored
- **GIVEN** an event titled `Ökobau`
- **WHEN** the user searches for `ökobau`
- **THEN** that event is a result

#### Scenario: Surrounding whitespace is ignored
- **GIVEN** an event titled `Neubau Müller`
- **WHEN** the user searches for `  Müller  `
- **THEN** that event is a result

#### Scenario: A title that does not contain the query is not a result
- **GIVEN** an event titled `Neubau Müller`
- **WHEN** the user searches for `Schmidt`
- **THEN** that event is not a result

### Requirement: Search scope
The system SHALL search the primary calendars of exactly the employees the grid is currently showing, and SHALL NOT read any absence calendar.
The searched range SHALL be one year before to one year after the current date, and the modal SHALL state that range.

#### Scenario: Assignments and bare events are searched
- **GIVEN** an assignment and a bare event whose titles both contain the query
- **WHEN** the search is run
- **THEN** both are results

#### Scenario: Absences are never results
- **GIVEN** an absence whose title contains the query
- **WHEN** the search is run
- **THEN** it is not a result

#### Scenario: Hidden employees are not searched
- **GIVEN** an employee the grid is hiding has an event whose title contains the query
- **WHEN** the search is run
- **THEN** that event is not a result

#### Scenario: Past and future weeks are searched
- **GIVEN** matching events in a week before and a week after the current week
- **WHEN** the search is run
- **THEN** both are results

#### Scenario: Events outside the range are not results
- **GIVEN** a matching event more than a year before the current date
- **WHEN** the search is run
- **THEN** it is not a result

#### Scenario: The searched range is stated
- **WHEN** the results are shown
- **THEN** the modal states the range that was searched

### Requirement: Result list
The system SHALL list the results oldest first, each naming its date, its weekday, the employee, the event's time and the event's title.
An all-day event SHALL be listed without a time rather than with an empty one.

#### Scenario: Results are ordered oldest first
- **GIVEN** matching events on several dates
- **WHEN** the results are shown
- **THEN** they are listed in ascending date order

#### Scenario: A result names its event
- **WHEN** a result is shown
- **THEN** it names the date, the weekday, the employee, the time and the title of the matching event

#### Scenario: An all-day result shows no time
- **GIVEN** a matching event without a start time
- **WHEN** its result is shown
- **THEN** it is listed without a time

#### Scenario: No match is reported
- **GIVEN** a query matching no event in the searched range
- **WHEN** the search completes
- **THEN** the modal reports in German that nothing was found, and states the range that was searched

### Requirement: Jumping to a result
The system SHALL show the week containing a chosen result and close the modal.

#### Scenario: Choosing a result shows its week
- **GIVEN** results are shown
- **WHEN** the user chooses one
- **THEN** the grid shows the week containing that result's date

#### Scenario: Choosing a result closes the modal
- **WHEN** the user chooses a result
- **THEN** the search modal is closed

#### Scenario: Choosing a result in the current week keeps the week
- **GIVEN** a result whose date falls in the week already shown
- **WHEN** the user chooses it
- **THEN** the grid still shows that week and the modal is closed

### Requirement: Partial failures
The system SHALL show the results it found when some employees' calendars could not be read, and SHALL name those employees in German.

#### Scenario: A failed calendar does not discard the other results
- **GIVEN** one employee's calendar cannot be read and another's holds a matching event
- **WHEN** the search is run
- **THEN** the matching event is shown as a result

#### Scenario: Failed employees are named
- **GIVEN** an employee's calendar could not be read
- **WHEN** the results are shown
- **THEN** the modal names that employee in a German message

#### Scenario: A search that fails entirely is reported
- **GIVEN** no employee's calendar can be read
- **WHEN** the search is run
- **THEN** the modal shows a German error message and no results
