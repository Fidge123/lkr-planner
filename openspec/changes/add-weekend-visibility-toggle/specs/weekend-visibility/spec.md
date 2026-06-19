## ADDED Requirements

### Requirement: Weekend visibility setting
The system SHALL provide a persisted display setting that controls whether the planning view includes Saturday and Sunday.
The setting SHALL default to off (weekend hidden) and SHALL be stored alongside the other display settings in the local store.

#### Scenario: Default value when unset
- **GIVEN** a local store with no explicit weekend visibility value
- **WHEN** the planning view loads the display settings
- **THEN** the weekend visibility setting resolves to off

#### Scenario: Persisted across restarts
- **GIVEN** the user has turned the weekend visibility setting on
- **WHEN** the application is restarted and the display settings are loaded
- **THEN** the weekend visibility setting is on

### Requirement: Planning view respects weekend visibility
The system SHALL render Monday to Friday when weekend visibility is off and Monday to Sunday when weekend visibility is on.

#### Scenario: Weekend hidden
- **GIVEN** the weekend visibility setting is off
- **WHEN** the planning view renders a week
- **THEN** exactly five day columns are shown, from Monday to Friday
- **AND** Saturday and Sunday are not shown

#### Scenario: Weekend shown
- **GIVEN** the weekend visibility setting is on
- **WHEN** the planning view renders a week
- **THEN** seven day columns are shown, from Monday to Sunday
- **AND** Saturday and Sunday appear after Friday

### Requirement: Toggle in settings dialog
The system SHALL expose the weekend visibility setting as a toggle in the settings dialog under the "Anzeige" section, labelled "Wochenende anzeigen".

#### Scenario: Turning the toggle on
- **GIVEN** the settings dialog is open with the weekend toggle off
- **WHEN** the user enables the weekend toggle and saves
- **THEN** the setting is persisted as on
- **AND** the planning view shows Saturday and Sunday

#### Scenario: Turning the toggle off
- **GIVEN** the settings dialog is open with the weekend toggle on
- **WHEN** the user disables the weekend toggle and saves
- **THEN** the setting is persisted as off
- **AND** the planning view shows only Monday to Friday
