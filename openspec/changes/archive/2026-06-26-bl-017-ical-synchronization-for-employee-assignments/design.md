## Context

With CalDAV as the source of truth (established in BL-015), assignment writes go directly to the employee's primary CalDAV calendar. There is no intermediate local store to sync from — writing to CalDAV IS the persistence operation.

## Goals / Non-Goals

**Goals:**
- Write lkr-planner assignments as VEVENT entries to employee primary CalDAV
- Ensure idempotent write behavior (no duplicate events via stable UID)
- Surface CalDAV write errors to the user immediately in German
- Never write to absence calendars

**Non-Goals:**
- Background sync queue (writes are inline with user action, errors are surfaced to UI)
- Sync status tracking per assignment (replaced by inline error handling)
- Reading assignments from CalDAV (covered by BL-015)

## Decisions

### Write Model: Inline on User Action
**Decision**: Write to CalDAV directly when the user saves or deletes an assignment (no background sync)
- No intermediate local store to sync from
- On save: PUT VEVENT to employee primary CalDAV; on delete: DELETE VEVENT from CalDAV
- On failure: surface German error message to the user; user can retry via the modal
- Eliminates sync queue, retry infrastructure, and per-record status tracking

### Event Format (follows BL-015 encoding)
**Decision**: Produce VEVENTs consistent with the BL-015 read format
- `SUMMARY`: Daylite project name (human-readable in any calendar app)
- `DESCRIPTION`: first line `daylite:/v1/projects/3001`, subsequent lines = optional user notes
- `DTSTART` / `DTEND`: daily time slots per BL-034 algorithm (fixed 08:00–16:00 window, split evenly)
- `UID`: stable UUID assigned at creation, never changed

### Idempotency via VEVENT UID
**Decision**: Use the assignment's stable UUID as the VEVENT UID
- UUID generated once at assignment creation, stored client-side with the assignment
- CalDAV PUT with an existing UID updates the event rather than duplicating it
- Repeated saves (e.g. on retry) produce exactly one event per assignment

### Absence Calendar Safety
**Decision**: Guard against writes to absence calendar URLs at the command level
- Before any PUT/DELETE, verify the target URL matches the employee's configured primary calendar
- Reject the operation if the URL matches the absence calendar URL
- Log rejected attempts for troubleshooting

## Risks / Trade-offs

- **Risk**: CalDAV write fails during assignment creation
  - **Mitigation**: Surface German error immediately; no partial state (no separate local store to diverge from CalDAV)

- **Risk**: Concurrent edit from another calendar client while the user has the modal open
  - **Mitigation**: Use CalDAV ETag for conflict detection on PUT; surface German conflict message if ETag mismatch
