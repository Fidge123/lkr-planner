## MODIFIED Requirements

### Requirement: Two-tier event display
The system SHALL distinguish lkr-planner assignments from bare calendar events.

#### Scenario: Display lkr-planner assignment
- **WHEN** a VEVENT has a DESCRIPTION first line matching `daylite:/<path>`
- **THEN** it is shown on a neutral card carrying a strip in its Daylite project's category color (`hex_colour`)
- **AND** an edit affordance is shown in the card's action area

#### Scenario: Assignment cards carry an action area
- **WHEN** an assignment card is rendered
- **THEN** its actions sit together on the right edge of the card as icon-only controls
- **AND** the card itself is not a control: the only things on it that can be triggered are the actions in that area

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
