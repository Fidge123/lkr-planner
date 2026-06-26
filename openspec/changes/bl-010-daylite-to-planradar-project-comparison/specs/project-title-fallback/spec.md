## ADDED Requirements

### Requirement: Project title fallback
The system SHALL determine the assignment display title using a fallback order that prefers the linked Planradar project name.

The fallback order is:
1. Custom name, if a custom-name concept exists and one is set (forward-compatible; out of scope until a custom-name feature is added)
2. Linked Planradar project name (resolved via the BL-010 link)
3. Daylite company name, only when the project has exactly one linked company
4. Daylite project name

#### Scenario: Planradar name preferred
- **GIVEN** an assignment whose Daylite project is linked to a Planradar project
- **WHEN** determining the display title
- **THEN** the Planradar project name is shown

#### Scenario: Single Daylite company fallback
- **GIVEN** an assignment with no linked Planradar project, but exactly one linked Daylite company
- **WHEN** determining the display title
- **THEN** the Daylite company name is shown

#### Scenario: Daylite project fallback
- **GIVEN** an assignment with no linked Planradar project and zero or multiple linked Daylite companies
- **WHEN** determining the display title
- **THEN** the Daylite project name is shown

#### Scenario: Custom name reserved as highest priority
- **WHEN** a custom-name feature later sets a custom name on an assignment
- **THEN** the custom name takes precedence over all other sources
- **AND** until that feature exists, resolution starts at the Planradar project name
