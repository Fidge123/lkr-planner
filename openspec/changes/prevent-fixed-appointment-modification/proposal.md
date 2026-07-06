## Why

Some CalDAV calendar entries are linked to Daylite projects that carry the category "Termin FIX geplant" (a fixed, already-confirmed appointment).
These must not be accidentally moved, retitled, or deleted from the planning grid, since doing so would silently break a commitment already made outside the app.

## What Changes

- Add a backend guard that rejects `update_assignment` and `delete_assignment` for any event linked to a Daylite project whose `category` is `"Termin FIX geplant"`, before the CalDAV write is issued.
- The guard determines the event's current linked project by fetching the event via its `href` and parsing the `daylite:/<path>` reference from its DESCRIPTION, then looking up that project's category via the Daylite API — it does not trust a client-supplied project reference, so it cannot be bypassed by the frontend.
- Extend the Daylite single-project lookup to also return `category` (it currently discards it).
- Disable the edit and delete affordances in the assignment modal when the currently loaded assignment's project category is `"Termin FIX geplant"` (using the category already available in the frontend's Daylite project cache), and show a German explanation instead of the usual controls.
- If a write is attempted anyway (e.g. a stale UI state) and the backend rejects it, show the German error message returned by the backend.
- `create_assignment` is unaffected — this only protects existing fixed appointments from being changed or removed.

## Capabilities

### New Capabilities
- `fixed-appointment-protection`: Backend guard that identifies events linked to a Daylite project with category "Termin FIX geplant" and rejects modification/deletion of those events.

### Modified Capabilities
- `ical-assignment-sync`: "Update assignment in CalDAV" and "Delete assignment from CalDAV" requirements gain a precondition that the operation is rejected when the target event is protected.
- `assignment-modal-crud`: "Edit existing assignment" and "Delete assignment" requirements gain a precondition that edit/delete affordances are disabled for protected assignments, with a German explanation shown instead.

## Impact

- `src-tauri/src/integrations/daylite/projects.rs`: extend `fetch_project_by_reference` (or add a sibling) to return `category` alongside name/status; add a `FIXED_APPOINTMENT_CATEGORY` constant alongside the existing `OVERDUE_CATEGORY`.
- `src-tauri/src/integrations/calendar/commands.rs`: `update_assignment` and `delete_assignment` fetch the current event (GET by `href`), parse its Daylite project reference, look up the project's category, and reject with a German error before issuing the CalDAV write.
- `src-tauri/src/integrations/calendar/events.rs`: extract the existing `daylite:/<path>` DESCRIPTION-parsing logic into a function reusable outside of full event classification, so the guard can call it directly on a freshly-fetched description.
- `src/app/components/assignment-modal.tsx`: disable edit/delete controls and show a German notice when the loaded assignment's project category is `"Termin FIX geplant"`.
- Tests: `cargo test` coverage for the guard (protected/unprotected/lookup-failure cases); `bun test` coverage for the modal's disabled state.
