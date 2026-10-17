# Spec Delta

## Purpose

Keep the planning grid's calendar data current without user action, so a planner returning to an app left open in the background sees the appointments as they are now instead of as they were when the week was last loaded.

## ADDED Requirements

### Requirement: Periodic refresh of the displayed week
The system SHALL fetch the assignments of the displayed week again once 5 minutes have passed since the last completed load of that week, as long as the application window is visible.

#### Scenario: Interval elapses while the window is visible
- **WHEN** the window is visible and 5 minutes have passed since the displayed week was last loaded
- **THEN** the assignments of the displayed week are fetched again

#### Scenario: Window is hidden
- **WHEN** the window is minimized or otherwise hidden
- **THEN** no periodic refresh is started while it stays hidden

#### Scenario: Other loads restart the interval
- **WHEN** the displayed week is fetched by navigation, by "Daten neu laden", by its "Erneut laden" action or after a write
- **THEN** the next automatic refresh is due 5 minutes after that fetch completes

#### Scenario: Showing a cached week
- **WHEN** the user navigates to a week whose assignments were fetched 3 minutes ago and are still cached
- **THEN** the next automatic refresh of that week is due 2 minutes later

### Requirement: Refresh on return to the window
The system SHALL fetch the assignments of the displayed week immediately when the window becomes visible or regains focus and the last completed load of that week is 5 minutes or more in the past.

#### Scenario: Returning after a long absence
- **WHEN** the window becomes visible or regains focus and the displayed week was last loaded 5 minutes or more ago
- **THEN** the assignments of the displayed week are fetched again without waiting for the next interval

#### Scenario: Returning shortly after a load
- **WHEN** the window regains focus and the displayed week was loaded less than 5 minutes ago
- **THEN** no additional fetch is started

#### Scenario: Returning after the computer slept
- **WHEN** the computer wakes from sleep and the window regains focus
- **THEN** the displayed week is fetched again if its last load is 5 minutes or more in the past

### Requirement: Automatic refresh discards other cached weeks
The system SHALL discard every cached week when an automatic refresh starts, so a week shown later is fetched again instead of being shown from data loaded before the refresh.

#### Scenario: Navigating to a previously visited week after a refresh
- **WHEN** an automatic refresh ran and the user then navigates to a week that was cached before it
- **THEN** that week's assignments are fetched again

### Requirement: Automatic refresh keeps the displayed data
The system SHALL keep the displayed week's assignments on screen while an automatic refresh is running and when it fails, and SHALL show the German error message with its "Erneut laden" action when it fails.

#### Scenario: Refresh in progress
- **WHEN** an automatic refresh is running
- **THEN** the grid keeps showing the previously loaded assignments
- **AND** the data loading indicator is shown

#### Scenario: Refresh succeeds
- **WHEN** an automatic refresh completes successfully
- **THEN** the grid shows the fetched assignments
- **AND** a previously shown error from an automatic refresh disappears

#### Scenario: Refresh fails
- **WHEN** an automatic refresh fails
- **THEN** the grid keeps showing the previously loaded assignments
- **AND** the German error message is shown with its "Erneut laden" action
- **AND** the next automatic refresh is due 5 minutes later

#### Scenario: Refresh during a drag
- **WHEN** an automatic refresh completes while an assignment is being dragged
- **THEN** the grid layout under the pointer stays unchanged until the drag ends
