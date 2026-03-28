## ADDED Requirements

### Requirement: ZEP admin credentials setup
The system SHALL allow the user to enter and securely store ZEP admin CalDAV credentials.

#### Scenario: Enter and save ZEP admin credentials
- **WHEN** user enters ZEP root URL, username, and password in the ZEP-Verbindung settings section
- **AND** clicks "Verbindung testen"
- **AND** credentials are valid (PROPFIND succeeds)
- **THEN** credentials are saved to macOS keychain
- **AND** German success message with discovered calendar count is shown

#### Scenario: Invalid ZEP admin credentials
- **WHEN** user enters ZEP credentials and clicks "Verbindung testen"
- **AND** server responds with 401 Unauthorized
- **THEN** credentials are NOT saved to keychain
- **AND** German error message explains authentication failure

#### Scenario: Missing ZEP admin credentials at test time
- **WHEN** user triggers "Speichern & Testen" for an employee calendar
- **AND** ZEP admin credentials are not found in keychain
- **THEN** connection test is not attempted
- **AND** German error message prompts user to configure ZEP credentials first

### Requirement: CalDAV calendar discovery
The system SHALL discover available calendars from the ZEP CalDAV server.

#### Scenario: Successful discovery on dialog open
- **WHEN** per-employee iCal dialog is opened
- **AND** admin credentials are present in keychain
- **THEN** a PROPFIND request is made to the ZEP CalDAV root URL
- **AND** the list of available calendars is shown in the calendar selector

#### Scenario: Discovery result cached for session
- **WHEN** per-employee iCal dialog is opened a second time in the same session
- **THEN** the previously discovered calendar list is reused without a new PROPFIND request

#### Scenario: Discovery fails on dialog open
- **WHEN** PROPFIND request fails (network error or auth failure)
- **THEN** calendar selector shows error state
- **AND** "Kalender neu laden" button is available
- **AND** German error message explains the failure

### Requirement: Calendar selection validation
The system SHALL require a calendar to be selected before saving or testing.

#### Scenario: No calendar selected
- **WHEN** user clicks "Speichern & Testen" with no calendar selected
- **THEN** action is rejected immediately
- **AND** German message prompts user to select a calendar

### Requirement: Independent calendar source testing
The system SHALL allow testing primary and absence calendars independently.

#### Scenario: Test primary assignment calendar
- **WHEN** user triggers "Speichern & Testen" for primary calendar
- **THEN** only the primary calendar is saved and tested
- **AND** result is independent of absence calendar state

#### Scenario: Test absence calendar
- **WHEN** user triggers "Speichern & Testen" for absence calendar
- **THEN** only the absence calendar is saved and tested
- **AND** result is independent of primary calendar state

### Requirement: Optional absence calendar
The system SHALL treat an unset absence calendar as intentional.

#### Scenario: Employee with no absence calendar
- **WHEN** `zep_absence_calendar` is None (unset)
- **THEN** no absence sync is performed for this employee
- **AND** no warning or error is shown for the unset absence calendar

#### Scenario: Non-ZEP employee
- **WHEN** both `zep_primary_calendar` and `zep_absence_calendar` are None
- **THEN** employee has no calendar integration
- **AND** ⚠ indicator is shown in timetable (primary not configured)

### Requirement: Combined save and test action
The system SHALL save the calendar assignment to Daylite before running the connection test.

#### Scenario: Save succeeds, test succeeds
- **WHEN** user selects a calendar and triggers "Speichern & Testen"
- **THEN** full calendar URL is constructed and saved to Daylite
- **AND** calendar name is saved to local store
- **AND** CalDAV connection test passes (GET with admin Basic Auth)
- **AND** success timestamp is stored
- **AND** German success message with timestamp is shown

#### Scenario: Save to Daylite fails
- **WHEN** Daylite is unreachable during save
- **THEN** local store is not modified
- **AND** connection test is not attempted
- **AND** German error message is shown

#### Scenario: Save succeeds, test fails
- **WHEN** calendar name is saved to Daylite but connection test fails
- **THEN** calendar name is persisted in local store
- **AND** failure timestamp is stored
- **AND** German error message with actionable hint is shown

### Requirement: Timestamp reset on calendar assignment change
The system SHALL clear the test timestamp when the calendar assignment changes.

#### Scenario: Calendar reassigned before testing
- **WHEN** user selects a different calendar from the dropdown
- **THEN** the previous test timestamp for that source is cleared
- **AND** the source shows "Nicht getestet" status until "Speichern & Testen" is triggered

### Requirement: German feedback with actionable hints
The system SHALL show German feedback with actionable error hints.

#### Scenario: Successful connection
- **WHEN** CalDAV calendar is accessible and returns valid iCal data
- **THEN** German success message is shown
- **AND** includes timestamp of test

#### Scenario: Authentication failure
- **WHEN** CalDAV server responds with 401 Unauthorized
- **THEN** German error "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen." is shown

#### Scenario: Calendar not found
- **WHEN** CalDAV server responds with 404 Not Found
- **THEN** German error "Kalender nicht gefunden. Kalender-Zuweisung prüfen." is shown

#### Scenario: Connection timeout
- **WHEN** CalDAV request times out
- **THEN** German error "Verbindung Zeitüberschreitung. Bitte Verbindung prüfen." is shown

#### Scenario: Invalid iCal response
- **WHEN** response does not contain valid iCal content
- **THEN** German error "Ungültige Antwort. Keine gültige iCal-Datei." is shown

### Requirement: Test timestamp persistence
The system SHALL persist the latest test timestamp per calendar source.

#### Scenario: Store test timestamp
- **WHEN** CalDAV test completes (success or failure)
- **THEN** timestamp is stored for that source in `EmployeeSetting`
- **AND** dialog displays "Zuletzt getestet: [timestamp]"

### Requirement: Timetable attention indicator
The system SHALL show an attention indicator in the timetable for employees whose primary calendar is not confirmed working.

#### Scenario: Primary calendar not assigned
- **WHEN** employee has no primary calendar configured (`zep_primary_calendar` is None)
- **THEN** ⚠ indicator is shown in the employee name cell

#### Scenario: Primary calendar untested
- **WHEN** employee has a primary calendar assigned but `primary_ical_last_tested_at` is None
- **THEN** ⚠ indicator is shown in the employee name cell

#### Scenario: Primary calendar last test failed
- **WHEN** the last connection test for the primary calendar failed
- **THEN** ⚠ indicator is shown in the employee name cell

#### Scenario: Primary calendar last test passed
- **WHEN** the last connection test for the primary calendar succeeded
- **THEN** no indicator is shown in the employee name cell

### Requirement: Per-employee iCal dialog
The system SHALL provide a per-employee dialog to view, assign, and test calendar sources.

#### Scenario: Open dialog from timetable
- **WHEN** user clicks the employee name cell
- **THEN** a dialog opens showing ZEP calendar configuration for that employee

#### Scenario: Dialog shows both sources independently
- **WHEN** dialog is open
- **THEN** primary calendar section and absence calendar section are both visible
- **AND** each section shows current calendar selection, last test status, and last tested timestamp
- **AND** each section has its own calendar selector and "Speichern & Testen" action

#### Scenario: Calendar selector populated from discovery
- **WHEN** dialog is open and discovery succeeded
- **THEN** each calendar selector shows all discovered ZEP calendars
- **AND** current assignment is pre-selected if one exists

### Requirement: Non-blocking validation
The system SHALL NOT block normal planning usage when tests fail.

#### Scenario: Failed test does not block planning
- **WHEN** CalDAV validation fails
- **THEN** user can still use the application
- **AND** planning continues with available data
