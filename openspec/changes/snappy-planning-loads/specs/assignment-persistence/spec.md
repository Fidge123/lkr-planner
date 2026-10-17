## MODIFIED Requirements

### Requirement: Daylite project resolution
The system SHALL resolve project details for lkr-planner events in the background after the week's events are displayed.

#### Scenario: Week is displayed before its projects are resolved
- **WHEN** the events of a week arrive from CalDAV
- **THEN** they are displayed without waiting for Daylite
- **AND** an assignment whose project is held in the resolution cache shows the cached name and category, even when the entry is stale
- **AND** an assignment whose project is not held in the cache shows its event SUMMARY as its title and is marked as pending

#### Scenario: Pending assignment is not shown as unresolved
- **WHEN** an assignment is marked as pending
- **THEN** its card shows no warning icon and no German note around its title

#### Scenario: Project resolved from Daylite
- **WHEN** a displayed week references a Daylite project that is not held in the resolution cache, or whose cache entry is stale
- **THEN** the system queries the Daylite API for the project details including its category in the background, without the loading spinner
- **AND** displays the resolved name and category on success

#### Scenario: Project served from the resolution cache
- **WHEN** a VEVENT references a project resolved recently enough that its cache entry is still valid
- **THEN** the name and category are displayed without a further API request

#### Scenario: Project resolution fails
- **WHEN** a VEVENT references a Daylite project
- **AND** the query for it does not succeed
- **AND** no earlier result for it is held in the resolution cache
- **THEN** the card keeps the event SUMMARY as its title and carries no project status
- **AND** a red warning icon marks the card
- **AND** the German note around the title, `"Beschreibung für "` and `" konnte nicht abgerufen werden"`, is shown in italics so the title stays distinguishable
- **AND** neutral color is used

## ADDED Requirements

### Requirement: Grid reflects own edits without a full reload
The system SHALL show the result of the user's own edit without reloading the whole week behind the loading spinner.

#### Scenario: Delete removes the card immediately
- **WHEN** the backend confirms that an assignment was deleted
- **THEN** its card is removed from the grid immediately

#### Scenario: Only the affected rows are refreshed
- **WHEN** an assignment is created, edited, deleted, or moved
- **THEN** the calendar events of only the affected employees are fetched again for the displayed week
- **AND** their rows are replaced without the loading spinner
- **AND** the rows of all other employees keep their data

#### Scenario: An older refresh does not undo a newer edit
- **WHEN** a refresh requested before a later optimistic edit returns
- **THEN** its result is discarded

### Requirement: Startup loads the displayed week once
The system SHALL read the local settings before the first week load and reload the week after the startup contact sync only when the sync changed a calendar.

#### Scenario: Contact sync leaves the calendars unchanged
- **WHEN** the startup contact sync completes
- **AND** every employee's primary and absence calendar equals the value read before the sync
- **THEN** the displayed week is not loaded again

#### Scenario: Contact sync changes a calendar
- **WHEN** the startup contact sync completes
- **AND** an employee's primary or absence calendar differs from the value read before the sync
- **THEN** the displayed week is loaded again
