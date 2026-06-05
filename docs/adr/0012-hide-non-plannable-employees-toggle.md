# ADR 0012: Hide Non-Plannable Employees Toggle

- Status: Accepted
- Date: 2026-05-29

## Context

The planning view previously fetched and showed only Daylite contacts with category "Monteur".
Two kinds of contacts clutter the view in practice: employees with no calendar configured (nothing can be planned for them) and test contacts (Daylite category "Test").
The planner wants a single toggle to hide both, on by default, while still being able to reveal everyone — including test employees — when needed.

An employee's calendar configuration lives on the Daylite contact `urls` and is mirrored into the local `employee_settings` (ADR 0008, ADR 0011), so the "has a calendar" signal is available without extra calls.

### Evaluated Options
- Fetch both "Monteur" and "Test" categories, filter to plannable employees in the frontend driven by a persisted toggle
  - Pros: One backend query (OR clause array) keeps Test employees available so the toggle can reveal them without a refetch; filtering uses data the frontend already has (category + mirrored calendar URL); toggle state persists like other settings.
  - Cons: Slightly more data fetched and a small amount of view-layer filtering logic.
- Filter entirely in the backend based on the toggle value
  - Pros: Frontend renders whatever it receives.
  - Cons: Toggling would require a refetch and round-trip; couples a view preference to the data-fetch layer; calendar-config lookups would have to move server-side per request.
- Keep fetching only "Monteur" and hide no-calendar employees in the frontend
  - Pros: Smallest change.
  - Cons: Cannot satisfy the requirement to optionally show test employees, because they would never be fetched.

## Decision

- Broaden the Daylite contact search to fetch both planning categories, "Monteur" and "Test", via a top-level OR clause array; rename the contact predicate to `is_planning_contact` / `filter_planning_contacts` to reflect this.
- Add a persisted `DisplaySettings.hideNonPlannableEmployees` flag to the local store, defaulting to `true` (hide), surfaced as a toggle in the settings dialog under a new "Anzeige" section.
- When the toggle is on, the planning view shows only plannable employees: category not "Test" and a configured primary calendar. When off, every fetched employee is shown.
  Filtering happens in the frontend (`filterVisibleEmployees`).

## Consequences

- Test employees and employees without a calendar are hidden by default, giving a clean planning view, but can be revealed instantly by turning the toggle off — no refetch required, since both categories are already loaded.
- The preference is device-local (stored in `local-store.json`) and defaults to hiding on a new device.
- The contact fetch returns more contacts than before; downstream code that assumed only "Monteur" contacts must account for "Test" contacts (the event-loading path naturally skips them, as test employees have no calendar).
