## Context

Employee iCal URLs are stored in Daylite contacts. Before using these URLs for synchronization, users need to validate that the URLs are accessible and return valid iCal data. URLs can be edited either in Daylite or directly in the app; Daylite remains the master record.

## Goals / Non-Goals

**Goals:**
- Validate iCal URL format before making network calls
- Test connection independently for primary and absence iCal
- Show German user feedback with actionable error messages
- Persist test timestamps for UI display
- Allow URL editing in-app with sync back to Daylite

**Non-Goals:**
- Employee CRUD operations against Daylite contacts
- Automatic periodic validation (manual trigger only)

## Decisions

### Validation Approach
**Decision**: Two-phase validation - format check first, then network test
- Reject obviously invalid URLs without network call
- Make HTTP HEAD request first, then GET if needed
- Parse response to verify valid iCal content (Content-Type: text/calendar or body starts with BEGIN:VCALENDAR)

### Error Messages
**Decision**: Map common errors to German messages with hints
- Connection timeout → "Verbindung Zeitüberschreitung. Bitte URL prüfen."
- SSL error → "SSL-Fehler. Zertifikat möglicherweise abgelaufen."
- Invalid response → "Ungültige Antwort. Keine gültige iCal-Datei."

### Store Schema
**Decision**: Extend `EmployeeSetting` inline with optional test timestamps (Option A)
- Add `primary_ical_last_tested_at: Option<String>` (ISO 8601)
- Add `absence_ical_last_tested_at: Option<String>` (ISO 8601)
- `absence_ical_url` remains a plain `String`; empty string = no absence sync (intentional)
- Fields are `Option`, so existing store files without them deserialize gracefully as `None`
- Changing a URL must clear its corresponding timestamp immediately

### Absence iCal is Optional
**Decision**: Primary iCal is the standard case; absence iCal is optional per employee
- Most employees share the same CalDAV server for both sources
- Some employees have only a primary iCal, possibly on a different CalDAV server
- Empty `absence_ical_url` is treated as intentional, not an error

### "Speichern & Testen" Flow
**Decision**: Save and test are a single combined action per iCal source
1. Validate URL format locally (instant, no network)
2. Save URL to Daylite (`daylite_update_contact_ical_urls`) — Daylite is master record
3. Save to local store, clear old timestamp for that source
4. Run HTTP connection test (`validate_ical_url` command)
5. Store result timestamp (success or failure)

Splitting save and test into separate actions is not supported; users who only want to save without testing must edit in Daylite directly.

### Daylite as Master Record
**Decision**: URL edits in-app are always synced back to Daylite immediately on save
- Prevents drift between local store and Daylite contacts
- Network dependency is accepted as part of the save action
- Failure to sync to Daylite aborts the flow before testing

### UX: Timetable Row Indicator
**Decision**: Show attention icon in employee name cell for any unconfirmed or failed state
- Icon visible when: no primary URL, primary URL untested, or last primary test failed
- Absence iCal issues shown only within the per-employee dialog, not in the timetable row
- No icon = primary iCal last test passed
- The entire employee name cell is clickable for all employees (not just those with issues)

### UX: Per-Employee Dialog
**Decision**: Single dialog with two independent sections — primary and absence
- "Einsatz-iCal" section always shown (URL input + "Speichern & Testen" + status)
- "Abwesenheits-iCal (optional)" section always shown; empty URL = no absence sync
- Each section shows: current URL, last test status, last tested timestamp (or "Nicht getestet")
- Sections operate independently; testing one does not affect the other
- Dialog is reachable by clicking anywhere in the employee name cell

## Risks / Trade-offs

- **Risk**: Network failures in testing
  - **Mitigation**: Show clear error, don't block planning

- **Risk**: iCal URL becomes invalid after validation
  - **Mitigation**: Timestamp shown in dialog; ⚠ icon reappears once timestamp is stale or missing

- **Risk**: Daylite sync fails during "Speichern & Testen"
  - **Mitigation**: Abort before test, show German error, local store unchanged

- **Risk**: Existing store files missing new timestamp fields
  - **Mitigation**: Fields are `Option<String>`; missing fields deserialize as `None` without error
