## Purpose

Let users choose between a light, dark or operating-system-driven color scheme.

## Requirements

### Requirement: Color scheme setting
The system SHALL provide a color scheme setting with the options light, dark and system.
The setting SHALL default to system and SHALL be stored in the browser local storage.

#### Scenario: Default value when unset
- **GIVEN** no color scheme is stored
- **WHEN** the application starts
- **THEN** the color scheme follows the operating system preference

#### Scenario: Persisted across restarts
- **GIVEN** the user has selected the dark color scheme
- **WHEN** the application is restarted
- **THEN** the dark theme is applied before the planning view renders

### Requirement: Color scheme selection in settings dialog
The system SHALL expose the color scheme setting in the settings dialog under the "Darstellung" section with the options "Hell", "Dunkel" and "System".

#### Scenario: Selecting a color scheme
- **GIVEN** the settings dialog is open on the "Darstellung" section
- **WHEN** the user selects "Dunkel"
- **THEN** the dark theme is applied immediately
- **AND** the selection is saved to local storage
