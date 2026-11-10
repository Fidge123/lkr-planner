## Purpose

Lets a planner create the Planradar project for a Daylite project from the planner, blank or as a copy of an existing project, and leaves it linked.

## ADDED Requirements

### Requirement: Create a blank project
The system SHALL offer `In Planradar anlegen` in the assignment edit modal when the link state of `planradar-project-linking` is unlinked or not found, and create a Planradar project from a form with name, address, start and end date and description.

#### Scenario: Form defaults
- **WHEN** the planner opens the form
- **THEN** the name holds the Daylite project name and the other fields are empty

#### Scenario: Submit
- **WHEN** the planner submits the form without a source project
- **THEN** a Planradar project with the entered values exists and is linked to the Daylite project

#### Scenario: Name already used
- **WHEN** the entered name equals the name of a Planradar project the account can access
- **THEN** nothing is created, and the planner is offered to link that project or change the name

### Requirement: Create a copy of an existing project
The system SHALL create a Planradar project as a copy of a source project picked from the project list, with the planner choosing which of details, groups, forms, users and layers are copied, all but users preselected.

#### Scenario: Copy and adjust
- **WHEN** the planner picks a source project, confirms name and aspects and the copy has finished
- **THEN** the copy is linked to the Daylite project
- **AND** a form shows the copy's address, dates and description, and saving it changes the copy

#### Scenario: Copy is in progress
- **WHEN** Planradar has accepted the copy but the copy does not exist yet
- **THEN** the modal shows that the project is being created and offers no second creation

#### Scenario: Copy does not appear in time
- **WHEN** the copy has not appeared when the wait ends
- **THEN** a German message says the copy may still appear in Planradar and can then be linked, and nothing is linked

### Requirement: Link the created project
The system SHALL store the link as soon as the Planradar project exists and never create a second project because storing the link failed.

#### Scenario: Link cannot be stored
- **WHEN** the project was created but Daylite rejects the link write
- **THEN** a German error message names the created project and offers `Erneut verknüpfen`, which stores the link without creating a project

#### Scenario: Adjusting the copy is abandoned
- **WHEN** the planner closes the form that adjusts a finished copy
- **THEN** the copy stays in Planradar and stays linked
