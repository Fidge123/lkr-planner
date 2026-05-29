# ADR 0011: Daylite as Source of Truth for Employee Calendar Configuration

- Status: Accepted
- Date: 2026-05-29

## Context

ADR 0008 established that an employee's assignment ("Einsatz iCal") and absence
("Abwesenheit iCal") calendar URLs are read from and written to the Daylite
contact `urls`. Saving a calendar for an employee writes the URL to Daylite and
also mirrors it into the device-local `employee_settings` in `local-store.json`
(see ADR 0005).

However, on startup the app only read calendar URLs from the local store and
never reconciled them back from Daylite. A calendar configured on one device was
therefore invisible on another device, whose local store starts empty: events
would not load and the configuration dialog showed no assigned calendar, even
though Daylite held the correct URLs.

### Evaluated Options
- Reconcile local `employee_settings` from Daylite contacts on every contact sync
  - Pros: Reuses the existing contact fetch, keeps the local store as a restart-safe cache and offline fallback, and requires no read-path changes in `load_week_events`.
  - Cons: Calendar URLs only become correct after contacts are synced, so a brief stale window can exist on first launch before the sync completes.
- Read calendar URLs directly from the Daylite cache at point of use
  - Pros: No duplicated state between the contact cache and `employee_settings`.
  - Cons: Larger refactor of `load_week_events` and the dialog, loses the simple per-employee settings record, and still needs cached contacts to work offline.
- Keep local-store-only configuration and require reconfiguring per device
  - Pros: No code change.
  - Cons: Fails the core requirement; configuration does not propagate across devices and contradicts Daylite being the source of truth.

## Decision

- Treat Daylite as the single source of truth for employee calendar
  configuration. The local `employee_settings` calendar URLs are a device-local
  mirror of Daylite, not an independent setting.
- Whenever fresh contacts are fetched from Daylite (`daylite_list_contacts`),
  reconcile each employee's `zep_primary_calendar` / `zep_absence_calendar` from
  the contact's managed `urls`:
  - A URL present in Daylite overrides the local value.
  - A managed URL absent in Daylite clears the local value.
  - When a URL changes, the corresponding connection-test result
    (`*_ical_last_tested_at` / `*_ical_last_test_passed`) is cleared because it
    no longer describes the current URL.
- On startup the frontend syncs Daylite contacts first (deduplicated with the
  planning grid's own fetch), then reads the reconciled local store and reloads
  the week's assignments, so calendars configured elsewhere appear without a
  manual refresh.

## Consequences

- A calendar configured on one device is picked up on every other device once
  Daylite contacts are synced.
- The local store remains a restart-safe cache and offline fallback; when Daylite
  is unreachable, the last reconciled values continue to be used.
- Editing the managed iCal `urls` directly in Daylite is reflected in the app on
  the next contact sync, including clearing a calendar by removing its URL.
