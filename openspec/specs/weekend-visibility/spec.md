## Purpose

Let the planning view optionally include Saturday and Sunday so teams that occasionally work weekends can plan and review those days, while keeping the uncluttered Monday-to-Friday view as the default.

## Requirements

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

### Requirement: Current-week anchoring is weekend-aware
The system SHALL anchor the current week (the default, non-navigated week) so that today is always visible.
When weekend visibility is on, the current week SHALL be the Monday-to-Sunday block that contains today, including when today is Saturday or Sunday.
When weekend visibility is off, the existing behavior is retained: on a Saturday or Sunday the current week anchors to the upcoming Monday, since the present weekend day cannot be shown anyway.

#### Scenario: Saturday with weekend on
- **GIVEN** the weekend visibility setting is on
- **AND** today is a Saturday
- **WHEN** the planning view loads the current (non-navigated) week
- **THEN** the week shown is the Monday-to-Sunday block that contains today
- **AND** today's Saturday column is visible without navigating to a previous week

#### Scenario: Sunday with weekend on
- **GIVEN** the weekend visibility setting is on
- **AND** today is a Sunday
- **WHEN** the planning view loads the current (non-navigated) week
- **THEN** the week shown is the Monday-to-Sunday block that contains today
- **AND** today's Sunday column is visible without navigating to a previous week

#### Scenario: Weekend off retains upcoming-Monday anchoring
- **GIVEN** the weekend visibility setting is off
- **AND** today is a Saturday or Sunday
- **WHEN** the planning view loads the current (non-navigated) week
- **THEN** the week shown starts on the upcoming Monday
- **AND** Monday to Friday of the upcoming week are shown

### Requirement: Toggle in settings dialog
The system SHALL expose the weekend visibility setting as a toggle in the settings dialog under the "Anzeige" section, labelled "Wochenende anzeigen".

#### Scenario: Turning the toggle on
- **GIVEN** the settings dialog is open with the weekend toggle off
- **WHEN** the user enables the weekend toggle and saves the dialog
- **THEN** the setting is persisted as on
- **AND** once the dialog is saved the planning view refreshes and shows Saturday and Sunday

#### Scenario: Turning the toggle off
- **GIVEN** the settings dialog is open with the weekend toggle on
- **WHEN** the user disables the weekend toggle and saves the dialog
- **THEN** the setting is persisted as off
- **AND** once the dialog is saved the planning view refreshes and shows only Monday to Friday
