## ADDED Requirements

### Requirement: Open a card's Daylite project in Daylite
The system SHALL offer an action on an assignment card in the planning grid that opens the card's Daylite project in the Daylite macOS app, bringing Daylite to the front and showing that project.

#### Scenario: Project opens in Daylite
- **WHEN** the planner triggers the action on an assignment card whose Daylite project resolved
- **THEN** the card's Daylite project reference is handed to the backend
- **AND** the system asks macOS to open the Daylite deep link for that project
- **AND** Daylite is launched if it is not running, or brought to the front if it is

#### Scenario: Daylite decides how the project is presented
- **WHEN** Daylite receives the deep link
- **THEN** how the project is shown, in a new tab or in the current one, is Daylite's behavior
- **AND** the system does not attempt to influence it

#### Scenario: Daylite is not installed
- **WHEN** no application is registered for the Daylite URL scheme
- **THEN** the system does not check for Daylite beforehand and does not show a message of its own
- **AND** the operating system reports the missing handler

### Requirement: Deep link URL contract
The system SHALL address a Daylite project by the numeric identifier carried in its Daylite reference.

#### Scenario: Reference is translated to a deep link
- **WHEN** a project reference of the form `/v1/projects/<id>` is opened
- **THEN** the URL `daylite4://ShowObject/Project/<id>` is opened

#### Scenario: Reference does not name a project
- **WHEN** the reference does not have the form `/v1/projects/<id>` with a numeric id
- **THEN** no URL is opened
- **AND** the failure is reported to the frontend as a German error message

### Requirement: Only cards with a resolved Daylite project offer the action
The system SHALL show the action exclusively on assignment cards whose Daylite project reference was resolved against Daylite.

#### Scenario: Resolved assignment
- **WHEN** an assignment card carries a Daylite project reference and the project resolved
- **THEN** the action is shown on the card

#### Scenario: Unresolved assignment
- **WHEN** an assignment card carries a Daylite project reference whose project could not be read, so the card shows the German placeholder note
- **THEN** no action is shown on the card

#### Scenario: Bare and absence events
- **WHEN** a card is a bare calendar event or an absence
- **THEN** no action is shown on the card

### Requirement: Card action area
The system SHALL place card actions in an area on the right edge of an assignment card, holding icon-only controls without a border or a background of their own.

#### Scenario: Appearance of the action
- **WHEN** the action is shown on an assignment card
- **THEN** it is an icon-only control on the right edge of the card
- **AND** it carries no border and no background of its own
- **AND** it carries a German accessible name naming what it opens

#### Scenario: Card title keeps its room
- **WHEN** an assignment card shows the action area
- **THEN** the card's title does not run underneath it
- **AND** a title too long for the remaining width still wraps and grows the card vertically

#### Scenario: Action area holds more than one action
- **WHEN** a further card action is added later
- **THEN** it takes its place in the same area alongside the existing one without the card being laid out differently

### Requirement: Card actions do not disturb card interaction
The system SHALL keep the card's own click and drag behavior reachable and unchanged where the action is shown.

#### Scenario: Triggering the action does not open the modal
- **WHEN** the planner triggers a card action
- **THEN** the edit modal does not open

#### Scenario: Triggering the action does not start a drag
- **WHEN** the planner presses and moves the pointer on a card action
- **THEN** no drag operation starts and the card stays in place

#### Scenario: The rest of the card behaves as before
- **WHEN** the planner clicks or drags the card outside the action area
- **THEN** the edit modal opens, or the drag starts, exactly as it does today

#### Scenario: Card being dragged
- **WHEN** a card is the one currently being dragged and is lifted out of the cell's flow
- **THEN** its action area is not visible either
