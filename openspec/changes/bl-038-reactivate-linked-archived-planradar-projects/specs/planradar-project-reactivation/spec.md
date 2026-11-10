## Purpose

Lets a planner reopen an archived Planradar project that a Daylite project is linked to, without leaving the planner.

## ADDED Requirements

### Requirement: Offer reactivation for an archived linked project
The system SHALL offer `Reaktivieren` in the assignment edit modal only when the link state of `planradar-project-linking` shows the linked project as archived.

#### Scenario: Archived project
- **WHEN** the planner opens the edit modal for an assignment whose linked Planradar project is archived
- **THEN** `Reaktivieren` is offered

#### Scenario: Active project
- **WHEN** the linked Planradar project is active
- **THEN** reactivation is not offered

### Requirement: Reactivate on request
The system SHALL reactivate the linked Planradar project only when the planner asks for it, and report success only after Planradar shows the project as active.

#### Scenario: Reactivation succeeds
- **WHEN** the planner chooses `Reaktivieren` and Planradar then shows the project as active
- **THEN** the modal shows the project as active and no longer offers reactivation

#### Scenario: Project was reactivated elsewhere
- **WHEN** the planner chooses `Reaktivieren` for a project that someone reactivated in the meantime
- **THEN** the modal shows the project as active without an error

#### Scenario: Reactivation fails
- **WHEN** Planradar rejects the request, cannot be reached or still shows the project as archived
- **THEN** a German error message names the cause, a missing permission included, and the project stays shown as archived
