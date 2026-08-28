## ADDED Requirements

### Requirement: Set a project's category
The system SHALL set a Daylite project's category to one of the categories Daylite offers for projects.
Creating a category, renaming one, and changing a category's color stay Daylite's business and are not offered here.

#### Scenario: Write the picked category
- **WHEN** a project's category is set to a category Daylite offers
- **THEN** the system sends `PATCH /projects/<id>` carrying that category's name
- **AND** the write is reported as successful only once Daylite accepted it

#### Scenario: Daylite rejects the write
- **WHEN** Daylite rejects setting the category
- **THEN** the normalized German error message is returned
- **AND** the project keeps the category it had

#### Scenario: The cached project follows the write
- **WHEN** a project's category has been set
- **THEN** the cached project for that reference carries the new category
- **AND** the next resolution of that reference returns it without waiting for the cache entry to expire

## MODIFIED Requirements

### Requirement: Category color retrieval
The system SHALL retrieve the Daylite categories a project can carry, with their colors, for coloring project events and for offering the categories a project can be given.

#### Scenario: Fetch project categories
- **WHEN** the project categories are requested
- **THEN** the system requests `GET /categories` with the `entity=project` filter
- **AND** parses `name`, `hex_colour`, and `is_active` for each category

#### Scenario: The categories are returned as a list
- **WHEN** the frontend requests the project categories
- **THEN** a list carrying each category's name, its color, and whether it is still active, sorted by name, is returned as a typed command result
- **AND** a failure yields no categories rather than a user-facing error

#### Scenario: One map serves the grid and the assignment modal
- **WHEN** the frontend colors an event or a project result by its category
- **THEN** the name-to-color map it reads is derived from that one list
- **AND** the planning grid, the project picker, and the category picker all read the categories from that same list

#### Scenario: Category without a color
- **WHEN** a category's `hex_colour` is null
- **THEN** the category is listed without a color
- **AND** events of projects in that category fall back to the neutral color

#### Scenario: Overdue results carry their category
- **WHEN** overdue projects are queried
- **THEN** each result carries the category `"Überfällig"` the query filtered on, even though Daylite omits the field from these minimal records

#### Scenario: Inactive categories keep their color
- **WHEN** a category has `is_active` set to false
- **AND** an existing project still references that category
- **THEN** the category's color is still used for that project's events
- **AND** the category is not offered as one a project can be given
