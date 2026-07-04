## Context

Absence event color is decided purely in the frontend, in `toCellEvent()` (`src/app/types.ts:39-55`), which currently assigns `bg-info/30` to every event with `kind === "absence"`.
Absence titles come from the iCal `SUMMARY` field with no structured category data — the `CalendarEventKind` enum only distinguishes `Assignment | Bare | Absence`, and no VEVENT `CATEGORIES` property is parsed anywhere in the CalDAV integration.
The user confirmed (via clarification) that classification should be based on keyword-matching the German title text, scoped to absence events only; assignment and bare event coloring are unaffected.
Daylite project-status colors already occupy `secondary`, `success`, `neutral`, `warning`, and `primary` (`projectStatusToColor`, `src/app/types.ts:19-36`), so absence categories must pick from the remaining DaisyUI semantic tokens (`info`, `accent`, `error`) to avoid visually colliding with assignment cards in the same grid.

## Goals / Non-Goals

**Goals:**
- Distinguish common absence categories (sick, special/training, vacation/other) by color in the timetable.
- Preserve current visual behavior for unmatched or vacation-titled absences (`bg-info/30`).
- Keep the classification a pure, testable frontend function with no backend changes.

**Non-Goals:**
- No iCal `CATEGORIES` property parsing or CalDAV/Zep upstream changes.
- No user-configurable category/color mapping — the keyword-to-color table is a fixed constant for this change.
- No changes to assignment or bare event coloring.

## Decisions

### Classification: keyword matching on title, frontend-only
Add a pure function (e.g. `absenceCategoryColor(title: string): string`) in `src/app/types.ts` that case-insensitively matches known German substrings against the absence title and returns a Tailwind/DaisyUI class.
Alternative considered: parsing iCal `CATEGORIES` in the Rust backend. Rejected per user decision — the upstream Zep/CalDAV source does not reliably set this field today, and title-based matching requires no backend or Zep changes.

### Color mapping table
| Keyword substrings (case-insensitive) | Category | Color class |
|---|---|---|
| `krank` | sick | `bg-error/30` |
| `sonderurlaub`, `fortbildung`, `schulung` | special leave / training | `bg-accent/30` |
| anything else (including `urlaub`) | vacation / other (default) | `bg-info/30` |

Alternative considered: a distinct color per every conceivable absence reason. Rejected as YAGNI — only `error` and `accent` are free, unused semantic tokens; adding more categories would require non-semantic custom colors, which is unnecessary for the requested scope.

### Spec/code alignment
The existing `employee-absence-display` spec says absences use "a warning color"; the code has always used `bg-info/30`. This change updates the spec text to match actual category-derived behavior instead of leaving the stale "warning" wording.

## Risks / Trade-offs

- [Keyword matching misses unanticipated title phrasing] → Mitigate by keeping the default fallback identical to today's uniform color, so unmatched titles degrade to current behavior rather than breaking.
- [Reusing `error` for sick absences could read as "something went wrong"] → Accepted trade-off given only 3 unused semantic tokens exist; documented here for future reviewers.

## Migration Plan

- Add the classifier function and wire it into `toCellEvent()`; no data migration needed since this is a pure rendering change.
- Update the `employee-absence-display` spec delta and archive it alongside the code change.

## Open Questions

None — scope and approach were confirmed via user clarification before writing this design.
