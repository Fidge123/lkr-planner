## 1. ZEP Admin Credentials

- [x] 1.1 Define keychain service/account identifiers for ZEP root URL, username, and password
- [x] 1.2 Implement Tauri command to save ZEP admin credentials to macOS keychain
- [x] 1.3 Implement Tauri command to load ZEP admin credentials from keychain
- [x] 1.4 Implement Tauri command to test ZEP admin credentials (PROPFIND on root URL with Basic Auth)
- [x] 1.5 Map credential test errors to German messages (401 → auth error, network → connection error)

## 2. CalDAV Calendar Discovery

- [x] 2.1 Implement PROPFIND request to ZEP CalDAV root URL with admin Basic Auth header
- [x] 2.2 Parse PROPFIND response to extract calendar names/paths
- [x] 2.3 Implement Tauri command `zep_discover_calendars` returning the list of available calendars
- [x] 2.4 Cache discovery result for the duration of the app session
- [x] 2.5 Map discovery errors to German messages (auth failure, network error, parse error)

## 3. Calendar Selection Validation

- [x] 3.1 Validate that a calendar name is selected (not None/empty) before "Speichern & Testen"
- [x] 3.2 Map validation failure to German message ("Bitte einen Kalender auswählen.")

## 4. CalDAV Connection Testing

- [x] 4.1 Implement CalDAV GET request with `Authorization: Basic` header for a given calendar
- [x] 4.2 Construct full calendar URL from root URL + calendar name at test time
- [x] 4.3 Add independent test for primary calendar (`zep_primary_calendar`)
- [x] 4.4 Add independent test for absence calendar (`zep_absence_calendar`)
- [x] 4.5 Parse response to verify iCal content (Content-Type: text/calendar or BEGIN:VCALENDAR in body)
- [x] 4.6 Map HTTP errors to German messages:
  - 401 → "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen."
  - 404 → "Kalender nicht gefunden. Kalender-Zuweisung prüfen."
  - timeout → "Verbindung Zeitüberschreitung. Bitte Verbindung prüfen."
  - invalid response → "Ungültige Antwort. Keine gültige iCal-Datei."

## 5. Store Schema & Timestamp Persistence

- [x] 5.1 Rename `primary_ical_url` → `zep_primary_calendar: Option<String>` in `EmployeeSetting`
- [x] 5.2 Rename `absence_ical_url` → `zep_absence_calendar: Option<String>` in `EmployeeSetting`
- [x] 5.3 Add `primary_ical_last_tested_at: Option<String>` to `EmployeeSetting`
- [x] 5.4 Add `absence_ical_last_tested_at: Option<String>` to `EmployeeSetting`
- [x] 5.5 Clear timestamp for a source when its calendar assignment changes
- [x] 5.6 Store test timestamp after each test (success or failure)
- [x] 5.7 Document migration: old `primary_ical_url`/`absence_ical_url` URL values cannot be migrated to calendar names automatically; employees must be remapped after update

## 6. "Speichern & Testen" Command

- [x] 6.1 Implement combined save-and-test Tauri command for a single calendar source
- [x] 6.2 Step 1: validate calendar selection is not None
- [x] 6.3 Step 2: construct full URL; sync to Daylite (`daylite_update_contact_ical_urls`), abort on failure
- [x] 6.4 Step 3: save calendar name to local store and clear timestamp
- [x] 6.5 Step 4: run CalDAV connection test with admin credentials from keychain
- [x] 6.6 Step 5: store result timestamp in local store

## 7. UI: ZEP Credentials Settings

- [x] 7.1 Add "ZEP-Verbindung" section to app settings (accessible from gear icon, alongside Daylite token)
- [x] 7.2 Input fields: ZEP root URL, ZEP admin username, ZEP admin password
- [x] 7.3 "Verbindung testen" button: triggers credential test, shows German success/error feedback
- [x] 7.4 On successful test: save credentials to keychain, show calendar count as confirmation
- [x] 7.5 Disable "Verbindung testen" while request is in flight

## 8. UI: Timetable Row Indicator

- [x] 8.1 Add clickable wrapper to employee name cell for all employees
- [x] 8.2 Show ⚠ icon when primary calendar is unset, untested, or last test failed
- [x] 8.3 No icon shown when last primary calendar test passed

## 9. UI: Per-Employee iCal Dialog

- [x] 9.1 Create dialog component with two independent sections (Einsatz / Abwesenheit)
- [x] 9.2 Trigger CalDAV discovery on dialog open; use session cache if available
- [x] 9.3 Each section: calendar selector (dropdown from discovered list) + "Speichern & Testen" button + status display
- [x] 9.4 Status display: success with timestamp, failure with German error + hint, or "Nicht getestet"
- [x] 9.5 Disable "Speichern & Testen" while request is in flight
- [x] 9.6 Clear section status when calendar selection changes (re-test required)
- [x] 9.7 Show absence section as optional (label + empty-state explains no absence sync)
- [x] 9.8 Show "Kalender neu laden" button when discovery failed or to force refresh

## 10. Testing

- [x] 10.1 Backend tests: credential save/load round-trip via keychain
- [x] 10.2 Backend tests: CalDAV discovery parses PROPFIND response correctly
- [x] 10.3 Backend tests: full URL construction from root URL + calendar name
- [x] 10.4 Backend tests: connection test with 401 response maps to correct German error
- [x] 10.5 Backend tests: Daylite sync failure aborts before connection test runs
- [x] 10.6 Backend tests: timestamp cleared when calendar assignment changes
- [x] 10.7 UI tests: ⚠ icon shown/hidden based on primary calendar state
- [x] 10.8 UI tests: dialog sections operate independently
- [x] 10.9 UI tests: in-flight state disables button; result shown after completion
- [x] 10.10 UI tests: selector disabled and error shown when discovery fails
