## Why

Absence events are currently all rendered with the same color (`bg-info/30`) regardless of the absence type, so users cannot tell vacation, sick leave, or special leave apart at a glance in the planning grid.
Distinguishing absence categories by color makes the weekly view scannable without opening each event.

## What Changes

- Classify absence events into categories (vacation, sick, special/training, other) by keyword-matching the event's title (iCal `SUMMARY`), since no structured category data exists in CalDAV/Zep today.
- Map each absence category to a distinct DaisyUI color class, reusing the existing `bg-<token>/30` styling convention, and avoiding colors already used for Daylite project status (`secondary`, `success`, `neutral`, `warning`, `primary`) to prevent visual ambiguity between assignment and absence cards.
- Unmatched or vacation-titled absences keep the current `bg-info/30` color, so existing behavior is preserved for the common case.
- Align the `employee-absence-display` spec wording with actual rendering behavior (the spec currently says "warning color" while the code uses `bg-info/30`).

## Capabilities

### Modified Capabilities
- `employee-absence-display`: "Absence event display" requirement changes from a single uniform absence color to a category-derived color, based on keyword-matching the absence title.

## Impact

- `src/app/types.ts`: extend `toCellEvent`/color derivation with a title-based absence category classifier.
- `openspec/specs/employee-absence-display/spec.md`: delta to the "Absence event display" requirement.
- No backend (Rust) changes required — absence titles already arrive via `EmployeeWeekEvents.events[].title`, and category-to-color mapping is a pure frontend display concern, matching the existing `projectStatusToColor` pattern.
- Tests: `bun test` coverage for the new classifier's keyword matches and fallback behavior.
