## Context

Daylite projects carry a `category: Option<String>` field (`src-tauri/src/integrations/daylite/projects.rs:23-26,50-53,83-86`), already used for filtering (`OVERDUE_CATEGORY = "Überfällig"`, lines 115-122).
Assignment write commands live in `src-tauri/src/integrations/calendar/commands.rs`: `create_assignment(app, employee_reference, date, project_ref, project_name)`, `update_assignment(app, href, uid, date, project_ref, project_name)`, and `delete_assignment(app, href)`.
Critically, `update_assignment` receives the *new/target* `project_ref` as a parameter, not the event's *current* one, and `delete_assignment` receives only `href` — neither command has the current project linkage available without a fetch.
`fetch_project_by_reference` (`daylite/projects.rs:411-437`) already fetches a single project by reference but discards `category`, returning only `(name, status)`.
The `daylite:/<path>` reference is parsed from an event's DESCRIPTION inside `classify_event` (`calendar/events.rs:12-58`), which operates on a fully-parsed `RawVEvent`, not a bare description string.
The existing "Absence calendar is never written" guard (`ical-assignment-sync` spec, enforced via `targets_absence_calendar` in `caldav.rs:141-147`) is the closest precedent for a pre-write rejection with a German error message.
The user confirmed the category source is the Daylite project the event is linked to (not an iCal field), and that enforcement must be a backend guard (not just a UI affordance), so a modified frontend cannot bypass it.

## Goals / Non-Goals

**Goals:**
- Reject `update_assignment` and `delete_assignment` before any CalDAV write when the event's linked Daylite project has category `"Termin FIX geplant"`.
- Make the guard authoritative: derive the current project link from the event itself (via CalDAV), not from a client-supplied parameter.
- Give the user an immediate, non-error-feeling UI cue (disabled controls) in the common case, with the backend guard as the enforcement backstop.

### The protected unit is the day, not the event

`update_assignment` and `delete_assignment` are rejected outright for a protected event, but `move_assignment` is only rejected when the target date differs from the event's current date, and `reorder_assignment` is not guarded at all.
Reassigning a fixed appointment to a colleague or resequencing it within its day does not break the commitment made outside the app; changing its day or removing it does.
Alternative considered: guard every write path unconditionally.
Rejected because it would freeze a fixed appointment against the ordinary planning the grid exists for, including the day re-slotting that any neighbouring write triggers.

**Non-Goals:**
- No change to `create_assignment` — creating new assignments is not restricted by this change.
- No configurable/multi-category protection list — a single fixed category string for this change (YAGNI; extend later if more categories need protection).
- No caching of the protection check result — each update/delete re-derives it fresh, since staleness here has real consequences (accidentally allowing a write that should be blocked).

## Decisions

### Guard fetches the event fresh rather than trusting a client-supplied project ref
`update_assignment`/`delete_assignment` will issue a CalDAV GET on `href` to read the event's current DESCRIPTION, parse the `daylite:/<path>` reference, and look up that project's category — before performing the PUT/DELETE.
Alternative considered: have the frontend pass the currently-known `project_ref` (already visible in the modal) as a parameter for the backend to check.
Rejected because the user explicitly required a guard that "cannot be bypassed by a modified frontend" — trusting a client-supplied value would defeat that.

### Extract description-parsing into a reusable function
Pull the `daylite:` prefix-stripping logic out of `classify_event` (`events.rs:12-58`) into a standalone function operating on a bare description string, callable both from event classification and from the new guard.
Alternative considered: duplicate the parsing logic in the guard.
Rejected as unnecessary duplication once extraction is straightforward.

### Extend `fetch_project_by_reference` to return category
Change its return type from `(name, status)` to include `category`, updating the one existing call site in `load_week_events` to ignore the new field.
Alternative considered: add a separate `fetch_project_category_by_reference` function.
Rejected — it would duplicate the same HTTP call `fetch_project_by_reference` already makes for the same project.

### Frontend disables affordances using the category the backend already resolved
`resolve_event` looks the linked project up per event and derives `category_color` from its category, so the category itself travels to the frontend as a new `project_category` field on `CalendarCellEvent`.
Disable edit/delete controls when it equals `"Termin FIX geplant"`, with a German explanatory notice.
Alternative considered: look the category up in a frontend cache of all Daylite projects.
Rejected because that cache no longer exists (the planning view rework removed `daylite-projects.ts`), and a list-all lookup could miss a project the server paginated away, silently disabling the notice.
This is advisory only — the backend guard is the actual enforcement, since the event data could be stale relative to what the backend independently determines per write.

### Error convention
Reuse the "Absence calendar is never written" pattern: reject before the network write, return a German message, e.g. `"Dieser Termin ist als 'Termin FIX geplant' gesperrt und kann nicht geändert oder gelöscht werden."`

## Risks / Trade-offs

- [Guard adds a CalDAV GET before every update/delete] → Accepted trade-off: correctness (cannot be bypassed) outweighs one extra round-trip on a low-frequency write path; the recently-proposed CalDAV event cache (see `caldav-caching-improvements`) may serve this GET from cache once implemented.
- [The event's resolved category and the backend guard could disagree if the category changes between the week load and the save] → Mitigate by making the backend guard authoritative and always re-derived fresh; the frontend disabled-state is just a UX hint.
- [Deleting a protected event that no longer resolves its project reference (e.g. project renamed/removed in Daylite)] → If the project lookup fails, treat as unprotected (fail open) so a broken Daylite link does not permanently lock an event; the user sees very obvious warnings so any changes are made knowing the risks.

## Migration Plan

- Implement the extracted parser and extended `fetch_project_by_reference` first (backwards compatible, additive).
- Wire the guard into `update_assignment`/`delete_assignment` behind `cargo test` coverage (red/green TDD).
- Wire the frontend disabled-state after the backend guard exists, so manual testing has the real rejection message to display when reached via a stale UI state.
- No data migration required; this is a behavior-only change.

## Open Questions

None — scope and enforcement approach were confirmed via user clarification before writing this design.
