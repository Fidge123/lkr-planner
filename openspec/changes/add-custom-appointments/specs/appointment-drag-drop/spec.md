# Spec Delta

## MODIFIED Requirements

### Requirement: Draggable assignment cards
The system SHALL allow assignment cards (`kind: "assignment"`) and custom appointment cards (`kind: "custom"`) in the planning grid to be picked up and dragged.

#### Scenario: Assignment card is draggable
- **WHEN** the user presses and drags an assignment card past the activation threshold
- **THEN** a drag operation starts carrying that assignment's identity (UID, href, source employee, source date)
- **AND** the source card is visually marked as being dragged

#### Scenario: Custom appointment card is draggable
- **WHEN** the user presses and drags a custom appointment card past the activation threshold
- **THEN** a drag operation starts carrying that appointment's identity and title
- **AND** dropping it follows the same rules as dropping an assignment card

#### Scenario: Bare and absence events are not draggable
- **WHEN** the user attempts to drag a bare or absence event card
- **THEN** no drag operation starts
- **AND** the card remains in place

#### Scenario: Assignments with an unresolved project are not draggable
- **WHEN** an assignment's Daylite project could not be resolved and its card shows the German placeholder text
- **THEN** the card is not draggable until the project data is available
- **AND** clicking the card still opens the edit modal

#### Scenario: Pressing the card without passing the drag threshold
- **WHEN** the user clicks an assignment card without moving past the activation threshold
- **THEN** the edit modal opens as before
