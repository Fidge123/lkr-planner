## Context

Employee calendar sources are managed through a single ZEP admin CalDAV account. The ZEP admin user has access to all employee calendars under one set of credentials. Each Daylite employee is mapped to their ZEP calendar(s) by name (calendar names in ZEP do not match Daylite employee names uniformly). The admin credentials are stored once in macOS keychain and reused for all employees.

Daylite contacts hold the derived full calendar URL (for interoperability). The local store holds the ZEP calendar name (the identifier used to select and construct the URL). Daylite remains the master record.

Non-ZEP employees (those without a ZEP calendar) have no calendar integration in this change. Custom/external iCal URL support is deferred to a future change.

## Goals / Non-Goals

**Goals:**
- Store ZEP admin credentials (root URL, username, password) securely in macOS keychain
- Discover available ZEP calendars via CalDAV PROPFIND using admin credentials
- Map discovered ZEP calendar names to Daylite employees per-employee
- Test CalDAV connection with admin credentials and persist result timestamps
- Show German user feedback with actionable error messages
- Allow re-mapping of calendar assignments in-app with sync back to Daylite

**Non-Goals:**
- Employee CRUD operations against Daylite contacts
- Automatic periodic validation (manual trigger only)
- Custom/external iCal URL support for non-ZEP employees (future change)
- Per-employee ZEP credentials (single admin credential only)

## Decisions

### ZEP Admin Credentials
**Decision**: Single ZEP admin account (root URL + username + password) stored in macOS keychain
- Admin credentials are entered once in a dedicated "ZEP-Verbindung" settings section
- Credentials stored via macOS Keychain API (not in local store JSON)
- Credentials are shared across all employees — no per-employee credential storage
- A connection test ("Verbindung testen") verifies credentials before saving to keychain
- If keychain entry is missing at test time, a clear German error is shown

### CalDAV Calendar Discovery
**Decision**: Fetch the list of available calendars via CalDAV PROPFIND on the root URL
- Discovery is triggered on-demand when the per-employee iCal dialog opens
- Result is cached for the duration of the app session (no repeated PROPFIND on every open)
- User can trigger a manual refresh via a "Kalender neu laden" button in the dialog
- Discovery failure (network error, auth failure) shows a German error; dialog still opens
- Discovered calendar list is used to populate a selector in the per-employee dialog

### Store Schema
**Decision**: Rename iCal URL fields to ZEP calendar name fields; add timestamp fields inline (Option A)
- `primary_ical_url` → `zep_primary_calendar: Option<String>` (ZEP calendar name/path, not a URL)
- `absence_ical_url` → `zep_absence_calendar: Option<String>` (ZEP calendar name/path)
- Add `primary_ical_last_tested_at: Option<String>` (ISO 8601 timestamp)
- Add `absence_ical_last_tested_at: Option<String>` (ISO 8601 timestamp)
- `Option<String>` fields: `None` = not configured; deserialization of missing fields yields `None`
- **Migration**: existing store files with `primary_ical_url`/`absence_ical_url` values cannot be automatically migrated (old values are URLs, new fields expect calendar names). On first load, old URL fields are ignored; employees must be remapped via CalDAV discovery.

### Absence Calendar is Optional
**Decision**: Primary calendar is the standard case; absence calendar is optional per employee
- `zep_absence_calendar = None` means no absence sync for that employee — not an error
- Employees with only one ZEP calendar leave absence unset intentionally
- "Abwesenheits-iCal (optional)" section in dialog reflects this with appropriate empty-state copy

### CalDAV URL Construction
**Decision**: Full calendar URL is constructed at runtime from root URL + calendar name
- Local store holds only the calendar name (e.g., `"John Doe - Einsatz"`)
- Full URL constructed as: `{zep_root_url}/{calendar_name}` (exact structure confirmed during implementation against real ZEP instance)
- Constructed URL is never stored in local store; it is derived on each access
- When saving to Daylite, the constructed full URL is written (Daylite stores URL, not calendar name)

### Connection Test Authentication
**Decision**: CalDAV connection test uses HTTP GET with Basic Auth header
- Admin credentials retrieved from keychain at test time
- `Authorization: Basic <base64(username:password)>` header attached to every request
- Response validation unchanged: `Content-Type: text/calendar` or body starts with `BEGIN:VCALENDAR`
- 401 Unauthorized → German error: "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen."
- 404 Not Found → German error: "Kalender nicht gefunden. Kalender-Zuweisung prüfen."

### "Speichern & Testen" Flow
**Decision**: Combined action — assign calendar, sync to Daylite, then test
1. Validate selection: `zep_primary_calendar` (or absence) is selected (not None)
2. Construct full URL from root URL + calendar name; save to Daylite (`daylite_update_contact_ical_urls`) — abort on Daylite failure
3. Save calendar name to local store, clear old timestamp for that source
4. Run CalDAV connection test (GET with admin Basic Auth)
5. Store result timestamp (success or failure)

Splitting save and test into separate actions is not supported. Users who only want to change the mapping without testing must re-trigger "Speichern & Testen" afterward.

### Daylite as Master Record
**Decision**: Calendar assignments in-app are always synced to Daylite immediately on save
- Daylite stores the full constructed CalDAV URL (for interoperability with other systems)
- Local store stores the ZEP calendar name (for display and selection)
- Failure to sync to Daylite aborts the flow before testing

### UX: ZEP Credentials Settings
**Decision**: Dedicated "ZEP-Verbindung" section in app settings (parallel to Daylite token)
- Fields: ZEP root URL, ZEP admin username, ZEP admin password
- "Verbindung testen" button validates credentials via PROPFIND before saving
- On success: credentials saved to keychain, calendar count shown as confirmation
- On failure: German error with hint shown; keychain not updated

### UX: Timetable Row Indicator
**Decision**: Show attention icon in employee name cell for any unconfirmed or failed state
- Icon visible when: no primary calendar assigned, primary calendar untested, or last primary test failed
- Absence calendar issues shown only within the per-employee dialog
- No icon = last primary calendar test passed
- Entire employee name cell is clickable for all employees

### UX: Per-Employee iCal Dialog
**Decision**: Single dialog with two independent sections — primary and absence
- "Einsatz-iCal" section always shown
- "Abwesenheits-iCal (optional)" section always shown; unset = no absence sync
- Each section: calendar selector (dropdown from discovered list) + "Speichern & Testen" button + status display
- Status display: success with timestamp, failure with German error + hint, or "Nicht getestet"
- If discovery failed, selector shows error state with "Kalender neu laden" option
- Dialog is reachable by clicking anywhere in the employee name cell

## Risks / Trade-offs

- **Risk**: ZEP admin credentials missing or invalid when connection test runs
  - **Mitigation**: Check keychain at dialog open; show actionable German error if missing

- **Risk**: CalDAV PROPFIND discovery fails (network or auth)
  - **Mitigation**: Show German error in dialog, allow manual retry; selector shows empty with reload option

- **Risk**: ZEP calendar URL structure unknown until implementation
  - **Mitigation**: Validate URL construction logic against real ZEP instance before finalizing; treat URL format as an implementation detail to confirm

- **Risk**: Field rename breaks existing local store files (migration gap)
  - **Mitigation**: Old `primary_ical_url`/`absence_ical_url` values are URLs — not calendar names — and cannot be migrated automatically. Employees must be remapped once. Document this in release notes.

- **Risk**: Network failures in connection testing
  - **Mitigation**: Show clear German error, do not block planning

- **Risk**: iCal calendar becomes inaccessible after validation
  - **Mitigation**: Timestamp shown in dialog; ⚠ icon reappears when timestamp is stale or missing

- **Risk**: Daylite sync fails during "Speichern & Testen"
  - **Mitigation**: Abort before test, show German error, local store unchanged
