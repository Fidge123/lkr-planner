## Why

Absence events are currently all rendered with the same color (`bg-info/30`) regardless of the absence type, so users cannot tell paid vacation, sick leave, or special leave apart at a glance in the planning grid.
Distinguishing absence categories by color makes the weekly view scannable without opening each event, and flagging a day where an appointment collides with an absence surfaces a scheduling conflict that currently has no visual signal at all.

Daylite assignment events have the same symptom for a different root cause: they all render in blue today, although Daylite itself shows each project in the color of its category.
The grid derives assignment color from the Daylite project status (`projectStatusToColor` in `src/app/types.ts`), but the assignment picker only offers projects with status `new_status` or `in_progress`, so every plannable event maps to `bg-primary` or `bg-secondary`, which are both blue hues in the light and the dark theme.
The backend additionally defaults every unknown or missing status to `new_status` (`map_project_status`), which also lands on blue.
The project's category is already fetched from the Daylite API (`DayliteProjectSummaryDto.category`) but is dropped before it reaches the calendar pipeline, so the frontend never sees the one attribute that carries the color users know from Daylite.

## What Changes

- Classify absence events into the six known Zep absence codes by keyword-matching the event's title (iCal `SUMMARY`): `UB` (paid vacation), `UU` (unpaid vacation), `KR` (sick), `Kro` (sick without pay), `SU` (special vacation), `FA` (time off in lieu).
- Group related codes under the same color hue at different intensities, and give unrelated codes their own hue:
  - Vacation family (`UB`, `SU`, `UU`) shares one hue at decreasing intensity.
  - Sick family (`KR`, `Kro`) shares a second hue at decreasing intensity.
  - `FA` (time off in lieu) is not vacation- or sickness-related, so it gets its own third hue.
- Add three new theme-level color tokens for these hues, validated for colorblind-safe separation from each other and from the existing Daylite project-status colors and the reserved conflict color (see design.md); unmatched/unrecognized absence titles keep the current `bg-info/30` color.
- Highlight any day where an absence event and an appointment (assignment event) occur together for the same employee with a red conflict indicator, paired with an icon and a German label (not color alone), since a scheduling conflict is a status, not a category.
- Align the `employee-absence-display` spec wording with actual rendering behavior (the spec currently says "warning color" while the code uses `bg-info/30`).
- Fetch Daylite categories with their colors in the Rust backend via `GET /categories?entity=project` (`name`, nullable `hex_colour`, `is_active`).
- Carry the resolved project's category color through the calendar event pipeline into `CalendarCellEvent` and color assignment events with it in the grid.
- Keep the existing status-derived color as the fallback for assignment events whose project has no category or whose category has no color, and keep the neutral placeholder color for unresolved projects.

## Capabilities

### Modified Capabilities
- `employee-absence-display`: "Absence event display" requirement changes from a single uniform absence color to a category-derived color per absence code, and gains a new conflict-highlighting requirement for days with both an absence and an appointment.
- `assignment-persistence`: "Two-tier event display" and "Daylite project resolution" requirements change from status-derived assignment colors to the Daylite category color, with the status color as fallback.
- `daylite-integration`: gains a new requirement for retrieving Daylite categories with their colors.

## Impact

- `src/app.css`: add three new theme-level custom color properties for the vacation, sick, and time-off-in-lieu hues (validated with the dataviz skill's palette checks).
- `src/app/types.ts`: extend `toCellEvent`/color derivation with a title-based absence category classifier covering all six codes, conflict detection when an absence and an assignment share a cell, and the category-color-with-status-fallback logic for assignment events.
- `src/app/components/timetable-cell.tsx`: render the conflict indicator (icon + German label) when a cell is flagged as conflicting, and render the assignment category color (dynamic hex value, so via inline style instead of a static Tailwind class).
- `openspec/specs/employee-absence-display/spec.md`: delta to the "Absence event display" requirement.
- `openspec/specs/assignment-persistence/spec.md`: delta to the "Two-tier event display" and "Daylite project resolution" requirements.
- `openspec/specs/daylite-integration/spec.md`: delta adding the category retrieval requirement.
- The absence colors and conflict detection need no backend (Rust) changes, since absence titles already arrive via `EmployeeWeekEvents.events[].title`.
- The assignment category colors do need backend changes: a categories fetch in `src-tauri/src/integrations/daylite`, threading the project category through `DayliteProjectCacheEntry`, `fetch_project_by_reference`, and `CalendarCellEvent` (`src-tauri/src/integrations/calendar`), plus regenerated TypeScript bindings.
- Tests: `bun test` coverage for the classifier's code matches, the fallback, conflict detection, and the assignment color fallback chain, plus `cargo test` coverage for the categories fetch and the category threading through event resolution.
