## ADDED Requirements

### Requirement: Highlight toggle on event cards
The system SHALL offer a highlight toggle among the action buttons of every assignment and bare event card.
Absence cards SHALL NOT offer it.
The toggle carries a German label naming what it matches, and reads as pressed while its card is highlighted.

#### Scenario: Assignment card offers the toggle
- **WHEN** a card for an assignment is rendered
- **THEN** its action buttons include a highlight toggle

#### Scenario: Bare event card offers the toggle
- **WHEN** a card for a bare event is rendered
- **THEN** its action buttons include a highlight toggle

#### Scenario: Absence card offers no toggle
- **WHEN** a card for an absence is rendered
- **THEN** it has no highlight toggle

#### Scenario: Active toggle reads as pressed
- **WHEN** a highlight is active
- **THEN** the toggle on every highlighted card reads as pressed to assistive technology

### Requirement: Matching by project reference or title
The system SHALL treat two events as the same work when they have the same kind and the same identifier for that kind.
An assignment is identified by its Daylite project reference, a bare event by its title compared after trimming, and an absence by nothing.
An assignment and a bare event SHALL NOT match each other.

#### Scenario: Assignments of the same project match
- **WHEN** the highlight is activated on an assignment
- **THEN** every assignment in the week with the same Daylite project reference is highlighted

#### Scenario: A renamed project still matches
- **GIVEN** a project was renamed in Daylite after some of its assignments were created
- **WHEN** the highlight is activated on one of its assignments
- **THEN** every assignment holding that project reference is highlighted regardless of the title shown on the card

#### Scenario: Assignments of different projects with the same name do not match
- **GIVEN** two Daylite projects have the same name
- **WHEN** the highlight is activated on an assignment of one of them
- **THEN** assignments of the other project are not highlighted

#### Scenario: An unresolved assignment matches on its reference
- **GIVEN** an assignment whose Daylite project could not be resolved, so its card shows the raw reference
- **WHEN** the highlight is activated on it
- **THEN** every assignment holding the same reference is highlighted, resolved or not

#### Scenario: Bare events with the same title match
- **WHEN** the highlight is activated on a bare event
- **THEN** every bare event in the week whose trimmed title is identical is highlighted

#### Scenario: A bare event does not match an assignment
- **GIVEN** a bare event whose title equals the name of a Daylite project scheduled in the same week
- **WHEN** the highlight is activated on that bare event
- **THEN** the assignments of that project are not highlighted

#### Scenario: Absences are never highlighted
- **WHEN** any highlight is active
- **THEN** no absence card is highlighted

### Requirement: Highlight scope
The system SHALL apply an active highlight to the cards of every employee row and every visible day of the week on screen.
Only cards SHALL change appearance.

#### Scenario: Matching cards across employees are highlighted
- **GIVEN** the same project is scheduled for several employees in the visible week
- **WHEN** the highlight is activated on one of its cards
- **THEN** the matching cards in every employee row are highlighted

#### Scenario: Matching cards across days are highlighted
- **GIVEN** the same project is scheduled on several days of the visible week
- **WHEN** the highlight is activated on one of its cards
- **THEN** the matching cards on every visible day are highlighted

#### Scenario: Nothing outside the cards changes
- **WHEN** a highlight is active
- **THEN** the day headers, the employee column, and the cell backgrounds are unchanged

### Requirement: One highlight at a time
The system SHALL keep at most one highlight active.
Activating the toggle on a card that is not highlighted replaces any active highlight with that card's.
Activating it on a card that is highlighted clears the highlight.

#### Scenario: Activating on another card replaces the highlight
- **GIVEN** a highlight is active for one project
- **WHEN** the user activates the toggle on a card of another project
- **THEN** only the cards of the other project are highlighted

#### Scenario: Activating on a highlighted card clears the highlight
- **GIVEN** a highlight is active
- **WHEN** the user activates the toggle on any highlighted card
- **THEN** no card is highlighted

### Requirement: Highlight lifetime
The system SHALL clear the highlight when the grid shows another week, and SHALL keep it across every other change to the grid.

#### Scenario: Week navigation clears the highlight
- **GIVEN** a highlight is active
- **WHEN** the user navigates to another week by arrow or by trackpad swipe
- **THEN** no card is highlighted

#### Scenario: Reloading assignments keeps the highlight
- **GIVEN** a highlight is active
- **WHEN** the assignments of the visible week are reloaded
- **THEN** the reloaded cards that match are highlighted

#### Scenario: A dragged card stays highlighted where it lands
- **GIVEN** a highlight is active
- **WHEN** the user drags a highlighted card to another day or employee in the same week
- **THEN** the card is highlighted in its new position

#### Scenario: A highlight matching nothing marks nothing
- **GIVEN** a highlight is active
- **WHEN** the last matching event of the visible week is deleted
- **THEN** no card is highlighted and the grid is otherwise unchanged

#### Scenario: The highlight is not persisted
- **WHEN** the application is restarted
- **THEN** no highlight is active

### Requirement: Highlight appearance
The system SHALL mark a highlighted card with a ring in a color reserved for highlighting, distinguishable from the drop-target and conflict rings and from the absence colors in the light and the dark theme.
The card's own colors, including the Daylite category strip, SHALL remain visible.

#### Scenario: Highlighted card is ringed
- **WHEN** a card matches the active highlight
- **THEN** it is rendered with the highlight ring

#### Scenario: The highlight ring is not the drop-target or conflict ring
- **GIVEN** a highlight is active
- **WHEN** a cell is a drop target or holds an absence conflict
- **THEN** the cell's ring and the cards' highlight ring are distinguishable

#### Scenario: Card colors survive the highlight
- **WHEN** a card is highlighted
- **THEN** its background color and its Daylite category strip are unchanged
