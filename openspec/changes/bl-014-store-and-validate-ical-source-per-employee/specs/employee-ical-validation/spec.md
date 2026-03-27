## ADDED Requirements

### Requirement: URL format validation
The system SHALL detect invalid URL format before making network calls.

#### Scenario: Invalid URL format
- **WHEN** user provides URL that is not a valid HTTP/HTTPS URL
- **THEN** validation fails immediately
- **AND** German error message explains the format issue

### Requirement: Independent iCal source testing
The system SHALL allow testing primary and absence iCal sources independently.

#### Scenario: Test primary assignment iCal
- **WHEN** user triggers "Speichern & Testen" for primary iCal URL
- **THEN** only the primary URL is saved and tested
- **AND** result is independent of absence iCal state

#### Scenario: Test absence iCal
- **WHEN** user triggers "Speichern & Testen" for absence iCal URL
- **THEN** only the absence URL is saved and tested
- **AND** result is independent of primary iCal state

### Requirement: Optional absence iCal
The system SHALL treat an empty absence iCal URL as intentional.

#### Scenario: Employee with no absence iCal
- **WHEN** absence iCal URL is empty
- **THEN** no absence sync is performed for this employee
- **AND** no warning or error is shown for the empty absence URL

#### Scenario: Employees may use different CalDAV servers
- **WHEN** primary and absence iCal URLs point to different hosts
- **THEN** each URL is tested independently against its own host
- **AND** no assumption is made that both URLs share the same server

### Requirement: Combined save and test action
The system SHALL save the URL to Daylite before running the connection test.

#### Scenario: Save succeeds, test succeeds
- **WHEN** user triggers "Speichern & Testen" with a valid, accessible URL
- **THEN** URL is saved to Daylite
- **AND** URL is saved to local store
- **AND** connection test passes
- **AND** success timestamp is stored
- **AND** German success message with timestamp is shown

#### Scenario: Save to Daylite fails
- **WHEN** Daylite is unreachable during save
- **THEN** local store is not modified
- **AND** connection test is not attempted
- **AND** German error message is shown

#### Scenario: Save succeeds, test fails
- **WHEN** URL is saved to Daylite but connection test fails
- **THEN** URL is persisted in local store
- **AND** failure timestamp is stored
- **AND** German error message with actionable hint is shown

### Requirement: Timestamp reset on URL change
The system SHALL clear the test timestamp when a URL is changed.

#### Scenario: URL changed before testing
- **WHEN** user edits an iCal URL
- **THEN** the previous test timestamp for that source is cleared
- **AND** the source shows "Nicht getestet" status until "Speichern & Testen" is triggered

### Requirement: German feedback with actionable hints
The system SHALL show German feedback with actionable error hints.

#### Scenario: Successful connection
- **WHEN** iCal URL is accessible and returns valid data
- **THEN** success message in German is shown
- **AND** includes timestamp of test

#### Scenario: Failed connection with actionable hint
- **WHEN** iCal URL test fails
- **THEN** German error message with specific hint is shown
- **AND** hint helps user understand how to fix the issue

### Requirement: Test timestamp persistence
The system SHALL persist the latest validation/test timestamp per iCal source.

#### Scenario: Store test timestamp
- **WHEN** iCal test completes (success or failure)
- **THEN** timestamp is stored for that source
- **AND** dialog displays "Zuletzt getestet: [timestamp]"

### Requirement: Timetable attention indicator
The system SHALL show an attention indicator in the timetable for employees whose primary iCal is not confirmed working.

#### Scenario: Primary iCal has no URL
- **WHEN** employee has no primary iCal URL configured
- **THEN** ⚠ indicator is shown in the employee name cell

#### Scenario: Primary iCal URL untested
- **WHEN** employee has a primary iCal URL but it has never been tested
- **THEN** ⚠ indicator is shown in the employee name cell

#### Scenario: Primary iCal last test failed
- **WHEN** the last connection test for the primary iCal URL failed
- **THEN** ⚠ indicator is shown in the employee name cell

#### Scenario: Primary iCal last test passed
- **WHEN** the last connection test for the primary iCal URL succeeded
- **THEN** no indicator is shown in the employee name cell

### Requirement: Per-employee iCal dialog
The system SHALL provide a per-employee dialog to view, edit, and test iCal sources.

#### Scenario: Open dialog from timetable
- **WHEN** user clicks the employee name cell
- **THEN** a dialog opens showing iCal configuration for that employee

#### Scenario: Dialog shows both sources independently
- **WHEN** dialog is open
- **THEN** primary iCal section and absence iCal section are both visible
- **AND** each section shows current URL, last test status, and last tested timestamp
- **AND** each section has its own "Speichern & Testen" action

### Requirement: Non-blocking validation
The system SHALL NOT block normal planning usage when tests fail.

#### Scenario: Failed test does not block planning
- **WHEN** iCal validation fails
- **THEN** user can still use the application
- **AND** planning continues with available data
