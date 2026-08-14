## Purpose

Let planners move between calendar weeks with a two-finger trackpad swipe on the planning grid, so browsing the plan feels continuous instead of requiring a trip to the navigation buttons in the header.

## Requirements

### Requirement: Horizontal swipe pulls in the neighbouring week
The system SHALL treat a horizontal two-finger swipe on the planning grid as a week gesture that pulls the neighbouring week in from the side the swipe comes from.
Swiping towards the left SHALL pull the next week in from the right edge, swiping towards the right SHALL pull the previous week in from the left edge.
The incoming week SHALL cover the current week rather than push it aside, and SHALL follow the fingers one to one until it has covered the grid completely.

#### Scenario: Pulling the next week in
- **GIVEN** the planning grid shows a week
- **WHEN** the user swipes horizontally towards the left
- **THEN** the next week is drawn over the grid from the right edge
- **AND** it moves with the swipe, exposing more of itself the further the swipe goes

#### Scenario: Pulling the previous week in
- **GIVEN** the planning grid shows a week
- **WHEN** the user swipes horizontally towards the right
- **THEN** the previous week is drawn over the grid from the left edge

#### Scenario: Vertical scrolling is untouched
- **GIVEN** the planning grid is taller than the window
- **WHEN** the user scrolls vertically, or diagonally with a mostly vertical motion
- **THEN** the grid scrolls as before
- **AND** no week is pulled in

### Requirement: The incoming week shows the events already loaded for it
The system SHALL render the incoming week from the assignments already cached for it, including the neighbouring weeks the planning view prefetches.
A week with no cached assignments SHALL be shown as an empty grid with that week's day headers, and SHALL fill in once its assignments arrive.

#### Scenario: Prefetched week
- **GIVEN** the assignments of the next week are already in the cache
- **WHEN** the user pulls that week in
- **THEN** its day headers and its assignment cards are visible while the gesture runs

#### Scenario: Week not loaded yet
- **GIVEN** the assignments of the next week are not cached
- **WHEN** the user pulls that week in
- **THEN** the week is shown with its day headers and without assignment cards
- **AND** the assignments appear once the load for that week finishes

#### Scenario: Prefetching never delays the active week
- **GIVEN** the planning view loads the week the user is on
- **WHEN** the neighbouring weeks are prefetched
- **THEN** the prefetch is dispatched only after the active week has loaded
- **AND** the prefetch is skipped when the user moved to another week meanwhile

### Requirement: A swipe past the commit threshold changes the week
The system SHALL complete the gesture once the swipe ends, using how far the week was pulled in as the decision.
A week pulled in by at least 20 percent of the grid width SHALL slide the rest of the way, cover the current week and become the displayed week.
A week pulled in by less than that SHALL slide back off the edge it came from, leaving the displayed week unchanged.

#### Scenario: Committed swipe
- **GIVEN** the user has pulled the next week in by at least 20 percent of the grid width
- **WHEN** the swipe ends
- **THEN** the next week slides all the way over the current one
- **AND** the planning view shows the next week afterwards

#### Scenario: Cancelled swipe
- **GIVEN** the user has pulled the next week in by less than 20 percent of the grid width
- **WHEN** the swipe ends
- **THEN** the pulled-in week slides back out of view
- **AND** the planning view still shows the same week

#### Scenario: One swipe changes one week
- **GIVEN** a swipe has just changed the week
- **WHEN** the trackpad's momentum keeps the wheel firing afterwards
- **THEN** no further week change is started until the input goes quiet

### Requirement: A grid wider than the window scrolls before it swipes
The system SHALL let a planning grid that does not fit the window scroll horizontally first, and SHALL start the week gesture only from a horizontal swipe that continues past the scroll edge it is heading for.
The incoming week SHALL be shown at the same horizontal offset as the grid it covers.

#### Scenario: Scrolling to the edge first
- **GIVEN** the planning grid is wider than the window and is not scrolled to its right edge
- **WHEN** the user swipes horizontally towards the left
- **THEN** the grid scrolls right as before
- **AND** no week is pulled in until the grid has reached its right edge

#### Scenario: Swiping from the edge
- **GIVEN** the planning grid is scrolled to its right edge
- **WHEN** the user keeps swiping towards the left
- **THEN** the next week is pulled in
- **AND** it shows the same columns as the grid it covers

### Requirement: An appointment drag keeps week navigation to itself
The system SHALL suppress the swipe gesture while an assignment card is being dragged, so the week only changes through the drag's own edge hovering.

#### Scenario: Swipe during a drag
- **GIVEN** the user is dragging an assignment card
- **WHEN** a horizontal swipe reaches the grid
- **THEN** no week is pulled in
- **AND** the drag keeps navigating weeks by hovering the window edge
