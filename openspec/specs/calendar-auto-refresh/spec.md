## Purpose

Keep the planning grid's calendar data current without user action, so a planner returning to an app left open in the background sees the appointments as they are now instead of as they were when the week was last loaded.

## Requirements

### Requirement: Periodic refresh of the displayed week
The system SHALL fetch the assignments of the displayed week again once 5 minutes have passed since the last completed load of that week, as long as the application window is visible.

#### Scenario: Interval elapses while the window is visible
- **WHEN** the window is visible and 5 minutes have passed since the displayed week was last loaded
- **THEN** the assignments of the displayed week are fetched again

#### Scenario: Window is hidden
- **WHEN** the window is minimized or otherwise hidden
- **THEN** no periodic refresh is started while it stays hidden

#### Scenario: Other loads restart the interval
- **WHEN** the displayed week is fetched by navigation, by "Daten neu laden" or by its "Erneut laden" action
- **THEN** the next automatic refresh is due 5 minutes after that fetch completes

#### Scenario: Row refresh after a write
- **WHEN** the rows of the affected employees are fetched again after a write
- **THEN** the time of the next automatic refresh is unchanged

#### Scenario: Showing a cached week
- **WHEN** the user navigates to a week whose assignments were fetched 3 minutes ago and are still cached
- **THEN** the next automatic refresh of that week is due 2 minutes later

### Requirement: Refresh on return to the window
The system SHALL check whether a refresh is due 5 seconds after the window becomes visible, regains focus or the network becomes available, and SHALL fetch the assignments of the displayed week at that point when the last completed load of that week is 5 minutes or more in the past.

#### Scenario: Returning after a long absence
- **WHEN** the window becomes visible or regains focus and the displayed week was last loaded 5 minutes or more ago
- **THEN** the assignments of the displayed week are fetched again 5 seconds later without waiting for the next interval

#### Scenario: Returning shortly after a load
- **WHEN** the window regains focus and the displayed week was loaded less than 5 minutes ago
- **THEN** no additional fetch is started

#### Scenario: Returning after the computer slept
- **WHEN** the computer wakes from sleep and the window becomes visible, regains focus or the network becomes available
- **THEN** the displayed week is fetched again 5 seconds later if its last load is 5 minutes or more in the past and the network is available

#### Scenario: Network returns while the window stays focused
- **WHEN** the network becomes available while the window keeps its focus and the displayed week was last loaded 5 minutes or more ago
- **THEN** the assignments of the displayed week are fetched again 5 seconds later

### Requirement: No automatic refresh while offline
The system SHALL NOT start an automatic refresh while the operating system reports no network connection.

#### Scenario: Refresh becomes due while offline
- **WHEN** a refresh becomes due while no network connection is available
- **THEN** no fetch is started
- **AND** no error is shown for the skipped refresh
- **AND** the refresh starts 5 seconds after the network becomes available

### Requirement: Automatic refresh discards other cached weeks
The system SHALL discard every cached week other than the displayed one when an automatic refresh starts, and SHALL fetch the displayed week instead of showing it from the cache, so a week shown later is fetched again instead of being shown from data loaded before the refresh.

#### Scenario: Navigating to a previously visited week after a refresh
- **WHEN** an automatic refresh ran and the user then navigates to a week that was cached before it
- **THEN** that week's assignments are fetched again

### Requirement: Automatic refresh keeps the displayed data
The system SHALL keep the displayed week's assignments on screen while an automatic refresh is running and when it fails.

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

#### Scenario: Refresh during a drag
- **WHEN** an automatic refresh completes while an assignment is being dragged
- **THEN** the grid layout under the pointer stays unchanged until the drag ends

#### Scenario: Edit while a refresh runs
- **WHEN** an assignment is moved or deleted while an automatic refresh runs
- **THEN** the result of that refresh does not undo the edit in the grid

### Requirement: Retry a failed automatic refresh once
The system SHALL retry a failed automatic refresh once after 30 seconds without showing an error, and SHALL show the German error message with its "Erneut laden" action only when the retry fails as well.

#### Scenario: Retry succeeds
- **WHEN** an automatic refresh fails and its retry 30 seconds later succeeds
- **THEN** the grid shows the fetched assignments
- **AND** no error is shown at any point

#### Scenario: Retry fails
- **WHEN** an automatic refresh fails and its retry 30 seconds later fails as well
- **THEN** the German error message is shown with its "Erneut laden" action
- **AND** the next automatic refresh is due 5 minutes after the failed retry

#### Scenario: Another load supersedes the retry
- **WHEN** the displayed week is loaded by navigation or a manual reload before a pending retry runs
- **THEN** the pending retry is dropped

#### Scenario: Retry falls due while offline
- **WHEN** the retry falls due while no network connection is available
- **THEN** no fetch is started and no error is shown
- **AND** the refresh starts 5 seconds after the network becomes available
