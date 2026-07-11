## ADDED Requirements

### Requirement: Category color retrieval
The system SHALL retrieve Daylite categories with their colors for coloring project events.

#### Scenario: Fetch project categories
- **WHEN** the system needs category colors for the planning grid
- **THEN** it requests `GET /categories` with the `entity=project` filter
- **AND** parses `name` and `hex_colour` for each category into a name-to-color map

#### Scenario: Category without a color
- **WHEN** a category's `hex_colour` is null
- **THEN** the category yields no color
- **AND** events of projects in that category fall back to the status-derived color

#### Scenario: Inactive categories keep their color
- **WHEN** a category has `is_active` set to false
- **AND** an existing project still references that category
- **THEN** the category's color is still used for that project's events
