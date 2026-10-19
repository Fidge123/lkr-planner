## MODIFIED Requirements

### Requirement: Draggable assignment cards
The system SHALL allow assignment cards (`kind: "assignment"`) in the planning grid to be picked up and dragged by their body, leaving the card's action controls free to be pressed.

#### Scenario: Assignment card is draggable
- **WHEN** the user presses and drags an assignment card's body past the activation threshold
- **THEN** a drag operation starts carrying that assignment's identity (UID, href, source employee, source date)
- **AND** the source card is visually marked as being dragged

#### Scenario: Bare and absence events are not draggable
- **WHEN** the user attempts to drag a bare or absence event card
- **THEN** no drag operation starts
- **AND** the card remains in place

#### Scenario: Assignments with an unresolved project are not draggable
- **WHEN** an assignment's Daylite project could not be resolved and its card shows the German placeholder text
- **THEN** the card is not draggable until the project data is available
- **AND** its edit action still opens the edit modal

#### Scenario: Pressing the card without passing the drag threshold
- **WHEN** the user presses an assignment card's body without moving past the activation threshold
- **THEN** nothing happens, because the card body is no longer a control
- **AND** the edit modal is reached through the card's edit action instead

#### Scenario: Pressing a card action never starts a drag
- **WHEN** the user presses and moves the pointer on one of the card's action controls
- **THEN** no drag operation starts and the card stays in place
