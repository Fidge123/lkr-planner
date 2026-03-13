## Context

Employee iCal URLs are stored in Daylite contacts. Before using these URLs for synchronization, users need to validate that the URLs are accessible and return valid iCal data.

## Goals / Non-Goals

**Goals:**
- Validate iCal URL format before making network calls
- Test connection independently for primary and absence iCal
- Show German user feedback with actionable error messages
- Persist test timestamps for UI display

**Non-Goals:**
- Employee CRUD operations against Daylite contacts
- Automatic periodic validation (manual trigger only)

## Decisions

### Validation Approach
**Decision**: Two-phase validation - format check first, then network test
- Reject obviously invalid URLs without network call
- Make HTTP HEAD request first, then GET if needed
- Parse response to verify valid iCal content

### Error Messages
**Decision**: Map common errors to German messages with hints
- Connection timeout → "Verbindung Zeitüberschreitung. Bitte URL prüfen."
- SSL error → "SSL-Fehler. Zertifikat möglicherweise abgelaufen."
- Invalid response → "Ungültige Antwort. Keine gültige iCal-Datei."

## Risks / Trade-offs

- **Risk**: Network failures in testing
  - **Mitigation**: Show clear error, don't block planning

- **Risk**: iCal URL becomes invalid after validation
  - **Mitigation**: Timestamp validation, show "last tested" in UI