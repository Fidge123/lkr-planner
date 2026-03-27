## 1. URL Validation

- [ ] 1.1 Implement URL format validation (HTTP/HTTPS check)
- [ ] 1.2 Reject invalid URLs before network call

## 2. Connection Testing

- [ ] 2.1 Implement HTTP request for iCal URL
- [ ] 2.2 Add independent test for primary iCal
- [ ] 2.3 Add independent test for absence iCal
- [ ] 2.4 Parse response to verify iCal content (check Content-Type: text/calendar or BEGIN:VCALENDAR in body)

## 3. German Error Messages

- [ ] 3.1 Map connection timeout to German message
- [ ] 3.2 Map SSL/certificate errors to German message
- [ ] 3.3 Map invalid response to German message
- [ ] 3.4 Add actionable hints for each error type

## 4. Store Schema & Timestamp Persistence

- [ ] 4.1 Add `primary_ical_last_tested_at: Option<String>` to `EmployeeSetting`
- [ ] 4.2 Add `absence_ical_last_tested_at: Option<String>` to `EmployeeSetting`
- [ ] 4.3 Clear timestamp for a source when its URL is changed
- [ ] 4.4 Store test timestamp after each test (success or failure)

## 5. "Speichern & Testen" Command

- [ ] 5.1 Implement combined save-and-test Tauri command for a single iCal source
- [ ] 5.2 Step 1: validate URL format
- [ ] 5.3 Step 2: sync URL to Daylite (`daylite_update_contact_ical_urls`), abort on failure
- [ ] 5.4 Step 3: save URL and clear timestamp in local store
- [ ] 5.5 Step 4: run HTTP connection test
- [ ] 5.6 Step 5: store result timestamp in local store

## 6. UI: Timetable Row Indicator

- [ ] 6.1 Add clickable wrapper to employee name cell for all employees
- [ ] 6.2 Show ⚠ icon when primary iCal has no URL, is untested, or last test failed
- [ ] 6.3 No icon shown when last primary iCal test passed

## 7. UI: Per-Employee iCal Dialog

- [ ] 7.1 Create dialog component with two independent sections (Einsatz / Abwesenheit)
- [ ] 7.2 Each section: URL input field, "Speichern & Testen" button, status display
- [ ] 7.3 Status display: success with timestamp, failure with German error + hint, or "Nicht getestet"
- [ ] 7.4 Disable "Speichern & Testen" while request is in flight
- [ ] 7.5 Clear section status when URL input changes (reflect that re-test is needed)
- [ ] 7.6 Show absence section as optional (label + empty state explains no absence sync)

## 8. Testing

- [ ] 8.1 Validation tests for allowed/disallowed URL formats
- [ ] 8.2 Backend tests for independent primary vs absence test execution
- [ ] 8.3 Backend tests: Daylite sync failure aborts before test runs
- [ ] 8.4 Backend tests: timestamp cleared when URL changes
- [ ] 8.5 UI tests: ⚠ icon shown/hidden based on primary iCal state
- [ ] 8.6 UI tests: dialog sections operate independently
- [ ] 8.7 UI tests: in-flight state disables button, result shown after completion
