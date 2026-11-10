## Purpose

Lets a planner see and set which Planradar project belongs to a Daylite project, so later Planradar steps act on the right project.

## ADDED Requirements

### Requirement: Show the link state
The system SHALL show in the assignment edit modal whether the assignment's Daylite project is linked to a Planradar project, and keep the assignment editable when that cannot be determined.

#### Scenario: Linked project
- **WHEN** the planner opens the edit modal for an assignment whose Daylite project carries a Planradar project ID
- **THEN** the Planradar project's name and whether it is active or archived are shown

#### Scenario: Unlinked or not found
- **WHEN** the Daylite project carries no Planradar project ID, or the stored ID matches no Planradar project the account can access
- **THEN** the modal says in German which of the two applies and offers `Mit Planradar-Projekt verknüpfen`

#### Scenario: Planradar not connected or unreachable
- **WHEN** no Planradar connection is configured or the Planradar request fails
- **THEN** the Planradar section shows a German message naming the cause
- **AND** saving and deleting the assignment work as before

### Requirement: Link an existing Planradar project
The system SHALL let the planner link the Daylite project to one Planradar project picked from all projects the account can access, archived ones marked as such, and store the link on the Daylite project.

#### Scenario: Pick and link
- **WHEN** the planner filters the Planradar project list by name, picks a project and confirms
- **THEN** the Planradar project ID is stored in the Daylite project's `planradar-link` custom field
- **AND** every device that loads the Daylite project shows it as linked

#### Scenario: Linked on another device in the meantime
- **WHEN** a link is confirmed for a Daylite project that another device linked to an existing Planradar project since the modal opened
- **THEN** the stored link is kept and shown, and the planner is told that the project was already linked

#### Scenario: Link cannot be stored
- **WHEN** Daylite rejects the write or cannot be reached
- **THEN** a German error message is shown, the link state is unchanged, and the planner can try again
