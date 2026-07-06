## Why

Absence events are currently all rendered with the same color (`bg-info/30`) regardless of the absence type, so users cannot tell paid vacation, sick leave, or special leave apart at a glance in the planning grid.
Distinguishing absence categories by color makes the weekly view scannable without opening each event, and flagging a day where an appointment collides with an absence surfaces a scheduling conflict that currently has no visual signal at all.

## What Changes

- Classify absence events into the six known Zep absence codes by keyword-matching the event's title (iCal `SUMMARY`): `UB` (paid vacation), `UU` (unpaid vacation), `KR` (sick), `Kro` (sick without pay), `SU` (special vacation), `FA` (time off in lieu).
- Group related codes under the same color hue at different intensities, and give unrelated codes their own hue:
  - Vacation family (`UB`, `SU`, `UU`) shares one hue at decreasing intensity.
  - Sick family (`KR`, `Kro`) shares a second hue at decreasing intensity.
  - `FA` (time off in lieu) is not vacation- or sickness-related, so it gets its own third hue.
- Add three new theme-level color tokens for these hues, validated for colorblind-safe separation from each other and from the existing Daylite project-status colors and the reserved conflict color (see design.md); unmatched/unrecognized absence titles keep the current `bg-info/30` color.
- Highlight any day where an absence event and an appointment (assignment event) occur together for the same employee with a red conflict indicator, paired with an icon and a German label (not color alone), since a scheduling conflict is a status, not a category.
- Align the `employee-absence-display` spec wording with actual rendering behavior (the spec currently says "warning color" while the code uses `bg-info/30`).

## Capabilities

### Modified Capabilities
- `employee-absence-display`: "Absence event display" requirement changes from a single uniform absence color to a category-derived color per absence code, and gains a new conflict-highlighting requirement for days with both an absence and an appointment.

## Impact

- `src/app.css`: add three new theme-level custom color properties for the vacation, sick, and time-off-in-lieu hues (validated with the dataviz skill's palette checks).
- `src/app/types.ts`: extend `toCellEvent`/color derivation with a title-based absence category classifier covering all six codes, plus conflict detection when an absence and an assignment share a cell.
- `src/app/components/timetable-cell.tsx`: render the conflict indicator (icon + German label) when a cell is flagged as conflicting.
- `openspec/specs/employee-absence-display/spec.md`: delta to the "Absence event display" requirement.
- No backend (Rust) changes required — absence titles already arrive via `EmployeeWeekEvents.events[].title`, and category-to-color mapping and conflict detection are pure frontend display concerns, matching the existing `projectStatusToColor` pattern.
- Tests: `bun test` coverage for the classifier's code matches, the fallback, and conflict detection.
