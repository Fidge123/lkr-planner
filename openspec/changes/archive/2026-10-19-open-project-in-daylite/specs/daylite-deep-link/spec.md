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
- **THEN** the URL `daylite://Command=ShowObject&Entity=Project&ID=<id>` is opened

#### Scenario: Reference does not name a project
- **WHEN** the reference does not have the form `/v1/projects/<id>` with a numeric id
- **THEN** no URL is opened
- **AND** the failure is reported to the frontend as a German error message

### Requirement: Only cards with a resolved Daylite project offer the action
The system SHALL show the Daylite action exclusively on assignment cards whose Daylite project reference was resolved against Daylite.

#### Scenario: Resolved assignment
- **WHEN** an assignment card carries a Daylite project reference and the project resolved
- **THEN** the Daylite action is shown alongside the card's edit action

#### Scenario: Unresolved assignment
- **WHEN** an assignment card carries a Daylite project reference whose project could not be read, so the card shows the German placeholder note
- **THEN** no Daylite action is shown
- **AND** the card's edit action is still shown, so the assignment stays editable

#### Scenario: Bare and absence events
- **WHEN** a card is a bare calendar event or an absence
- **THEN** no Daylite action is shown on the card

### Requirement: Appearance and placement of the Daylite action
The system SHALL present the Daylite action as one of the icon-only controls in the assignment card's action area.

#### Scenario: Appearance of the action
- **WHEN** the Daylite action is shown on an assignment card
- **THEN** it is an icon-only control carrying no border and no background of its own
- **AND** it carries a German accessible name naming what it opens

#### Scenario: Order within the action area
- **WHEN** an assignment card shows both actions
- **THEN** the edit action comes first and the Daylite action after it, in the same order on every card

#### Scenario: Card title keeps its room
- **WHEN** an assignment card shows its action area
- **THEN** the card's title does not run underneath the actions
- **AND** a title too long for the remaining width still wraps and grows the card vertically

### Requirement: The Daylite action is isolated from the card's other behavior
The system SHALL keep triggering the Daylite action from causing anything else the card does.

#### Scenario: Opening Daylite does not open the modal
- **WHEN** the planner triggers the Daylite action
- **THEN** the edit modal does not open

#### Scenario: Opening Daylite does not start a drag
- **WHEN** the planner presses and moves the pointer on the Daylite action
- **THEN** no drag operation starts and the card stays in place

#### Scenario: Card being dragged
- **WHEN** a card is the one currently being dragged and is lifted out of the cell's flow
- **THEN** its action area is not visible either
