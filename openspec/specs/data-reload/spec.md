## Purpose

Let planners pull fresh calendar and Daylite data on demand from the application menu, without restarting the app, when someone else changed a project or an appointment.

## Requirements

### Requirement: Reload data menu item
The system SHALL offer a "Daten neu laden" item in the window menu under File.

#### Scenario: Menu item is available
- **WHEN** the application window is open
- **THEN** the File menu contains the item "Daten neu laden"

### Requirement: Reload refreshes calendar and Daylite data
The system SHALL discard the cached calendar and Daylite data and fetch it again when the reload item is chosen.

#### Scenario: Choosing the menu item
- **WHEN** the user chooses "Daten neu laden"
- **THEN** the assignments of the displayed week are fetched again
- **AND** the employee contacts are fetched again, bypassing their cache
- **AND** the project category colors are fetched again on their next use
- **AND** the employee and display settings are read again from the local store

#### Scenario: Failed reload surfaces its error
- **WHEN** a reload fetch fails
- **THEN** the German error message of that fetch is shown with its "Erneut laden" action
