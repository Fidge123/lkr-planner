## Purpose

Make CalDAV the live source of truth for the planning grid by loading each employee's calendar events per week, resolving the Daylite project behind each assignment, and moving assignments between employee calendars.

## Requirements

### Requirement: Load assignments from CalDAV
The system SHALL load assignment events from each employee's primary CalDAV calendar.

#### Scenario: Load assignments for week
- **WHEN** user navigates to a week
- **THEN** VEVENTs for that week are fetched from each employee's primary CalDAV calendar
- **AND** displayed in the planning grid

#### Scenario: No events exist for week
- **WHEN** loading events for a week with no calendar entries
- **THEN** empty cells are shown
- **AND** user can create new assignments (via BL-016)

#### Scenario: Employee has no primary calendar configured
- **WHEN** an employee has no `zepPrimaryCalendar` setting
- **THEN** their row shows empty cells without triggering a fetch or error

### Requirement: Two-tier event display
The system SHALL distinguish lkr-planner assignments from bare calendar events.

#### Scenario: Display lkr-planner assignment
- **WHEN** a VEVENT has a DESCRIPTION first line matching `daylite:/<path>`
- **THEN** it is shown on a neutral card carrying a strip in its Daylite project's category color (`hex_colour`)
- **AND** an edit affordance is shown in the card's action area

#### Scenario: Assignment cards carry an action area
- **WHEN** an assignment card is rendered
- **THEN** its actions sit together in one area of the card as icon-only controls in a fixed order
- **AND** the card itself is not a control: the only things on it that can be triggered are the actions in that area

#### Scenario: Action area in a column wide enough for it
- **WHEN** the card's column leaves the title useful room beside the actions
- **THEN** the action area sits on the right edge of the card

#### Scenario: Action area in a narrow column
- **WHEN** the column is too narrow for that
- **THEN** the action area moves under the card's times, on its left edge
- **AND** the title keeps the full height of the card beside both

#### Scenario: The edit affordance is always available on an assignment
- **WHEN** an assignment card is rendered, including one whose Daylite project could not be resolved
- **THEN** the edit action is shown
- **AND** an assignment stays editable and deletable regardless of whether Daylite could be reached

#### Scenario: Category color is applied verbatim
- **WHEN** an assignment event is rendered with a category color
- **THEN** the value from Daylite is used as-is, so any CSS color notation it uses still applies
- **AND** it colors only the strip, leaving the card surface and its text unchanged

#### Scenario: Assignment without a category color
- **WHEN** a resolved Daylite project has no category, or its category has no color
- **THEN** the strip keeps its default muted color, the same one for every project status

#### Scenario: Category color is joined where the card is rendered
- **WHEN** an assignment card is rendered
- **THEN** its strip color comes from looking the project's category up in the category color map
- **AND** the map is read once for the whole session, so a recolored category appears after a reload or a restart

#### Scenario: Bare events carry no strip
- **WHEN** an event has no Daylite project reference
- **THEN** it is rendered without a strip
- **AND** it shares the same surface color as an assignment, so the strip alone marks an event as an assignment

#### Scenario: Display bare event
- **WHEN** a VEVENT has no structured Daylite project reference
- **THEN** it is shown with neutral/grey styling
- **AND** no action area and no edit affordance are shown (read-only)
- **AND** covers legacy manually-created events and employee blockers

#### Scenario: Display event start and end time
- **WHEN** a VEVENT has a start time (non-all-day event)
- **THEN** the start time is shown in HH:MM format on the left of the event card
- **AND** the end time is shown below the start time if present
- **AND** all-day events show no time

#### Scenario: Event card hover feedback
- **WHEN** the user hovers over any event card
- **THEN** a visual hover indicator is shown

#### Scenario: Card action hover feedback
- **WHEN** the user hovers over one of an assignment card's actions
- **THEN** that action is marked as the thing under the pointer, distinctly from the card's own hover indicator

#### Scenario: Long event titles
- **WHEN** an event title exceeds the card width
- **THEN** the title wraps to multiple lines
- **AND** the card grows vertically to accommodate the text

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

### Requirement: Week navigation with live data
The system SHALL use CalDAV as the data source for all week navigation.

#### Scenario: Navigate between weeks
- **WHEN** user navigates to a different week
- **THEN** CalDAV is queried for the new week's date range
- **AND** assignments for the new week are displayed

#### Scenario: Pre-fetch adjacent weeks
- **WHEN** a week is loaded
- **THEN** the previous and next weeks are silently pre-fetched into the cache
- **AND** navigation to an adjacent week displays instantly without a loading state

### Requirement: Loading and error states
The system SHALL maintain German loading and error states.

#### Scenario: Show loading state
- **WHEN** CalDAV events or the employee contacts are being fetched
- **THEN** a spinner with the German text "Daten werden geladen..." is shown in the header next to the settings button
- **AND** the planning table itself carries no loading text

#### Scenario: Show error state on per-employee fetch failure
- **WHEN** a CalDAV fetch fails for an individual employee
- **THEN** their row shows a German error indicator: "Kalender nicht verfügbar"
- **AND** a retry button is shown that re-fetches the week for all employees

#### Scenario: Show error state on total fetch failure
- **WHEN** the calendar data cannot be loaded at all (store unavailable, bad date)
- **THEN** a German error banner is displayed above the planning table
- **AND** user can retry the fetch

### Requirement: Move assignment between calendars
The system SHALL move an assignment from one employee's CalDAV calendar to another employee's CalDAV calendar in a single operation and report whether the move completed fully.

#### Scenario: Move to another employee's calendar
- **WHEN** a move is requested with the source assignment href and a target employee reference and date
- **THEN** a new VEVENT carrying the same project reference and project name is created on the target employee's primary calendar at the target date with the standard assignment time window
- **AND** the original VEVENT is deleted from the source calendar
- **AND** a result indicating a full move with the new CalDAV href is returned

#### Scenario: Source delete fails after target create
- **WHEN** the target VEVENT is created but deleting the source VEVENT fails
- **THEN** the source VEVENT is left in place
- **AND** a result is returned indicating a partial move, carrying both the new href and the source href
- **AND** the operation does not report a plain success

#### Scenario: Target employee has no primary calendar
- **WHEN** a move targets an employee without a configured primary calendar
- **THEN** the operation fails with a German error message
- **AND** the source assignment is left untouched

#### Scenario: Refuse moves into an absence calendar
- **WHEN** a move would write into a configured absence calendar
- **THEN** the operation is refused with a German error message
- **AND** the source assignment is left untouched

#### Scenario: Target create fails
- **WHEN** creating the VEVENT on the target calendar fails
- **THEN** the source VEVENT is not deleted
- **AND** a German error message is returned

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
