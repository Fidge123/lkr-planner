## Context

Absence event color is decided purely in the frontend, in `toCellEvent()` (`src/app/types.ts:39-55`), which currently assigns `bg-info/30` to every event with `kind === "absence"`.
Absence titles come from the iCal `SUMMARY` field with no structured category data — the `CalendarEventKind` enum only distinguishes `Assignment | Bare | Absence`, and no VEVENT `CATEGORIES` property is parsed anywhere in the CalDAV integration.
The user confirmed the six real Zep absence codes: `UB` (paid vacation), `UU` (unpaid vacation), `KR` (sick), `Kro` (sick without pay), `SU` (special vacation), `FA` (time off in lieu) — and that classification is keyword-matching on the title, scoped to absence events only.
The user also requires: any day where an absence and an appointment (assignment event) coincide for the same employee must be highlighted red, and otherwise each code needs a color, with similar colors for similar types.
Daylite project-status colors already occupy the DaisyUI `secondary`, `success`, `neutral`, `warning`, and `primary` tokens (`projectStatusToColor`, `src/app/types.ts:19-36`), and `error` is reserved exclusively for the new conflict indicator, so it cannot double as a category color.
`accent` and `info` are the only other existing DaisyUI tokens, but the app's built-in dark theme ("business") reassigns hues inconsistently between light and dark for several tokens — most notably `accent` is teal (hue ≈185) in the light "corporate" theme but orange (hue ≈36) in the dark "business" theme, dangerously close to `error`'s dark hue (≈30). Reusing `accent` for a category color would therefore risk visually colliding with the conflict-red indicator in dark mode. This rules out reusing any existing semantic token for absence categories beyond the current `info` fallback.

Assignment (Daylite) events are all blue for a different root cause, confirmed by code investigation.
`toCellEvent()` colors assignments via `projectStatusToColor` (`src/app/types.ts:19-36`), the assignment picker only offers projects with status `new_status` or `in_progress` (`assignment-modal-filter` spec), and `map_project_status` (`src-tauri/src/integrations/daylite/projects.rs`) defaults every unknown or missing status to `new_status`.
`new_status` maps to `bg-primary` (blue, hue ≈242 light / ≈251 dark) and `in_progress` maps to `bg-secondary` (desaturated blue, hue ≈257 light / ≈229 dark), so every assignment in the grid renders in one of two blue tones.
Daylite itself colors projects by their category, and the API exposes those colors via `GET /categories` (fields `name`, nullable `hex_colour`, `is_active`, filterable with `entity=project`).
The project `category` name is already part of `DayliteProjectSummaryDto` but is dropped on the way to the grid: `DayliteProjectCacheEntry` stores only reference, name, and status, `fetch_project_by_reference` returns only `(name, status)`, and `CalendarCellEvent` has no category field.
The project cache (`daylite_cache.projects`) is additionally never written by any code path today, so every assignment resolves through the API fallback path.

## Goals / Non-Goals

**Goals:**
- Give each of the six absence codes a color, with the three related pairs/groups sharing a hue at different intensities.
- Reserve a red conflict indicator, paired with an icon and label (not color alone), for days where an absence and an appointment coincide.
- Keep all category and conflict colors colorblind-safe and consistent across the light and dark themes.
- Preserve current visual behavior (`bg-info/30`) for absence titles that don't match any known code.
- Color assignment events with their Daylite category color so the grid matches what users see in Daylite, with the current status colors as fallback.

**Non-Goals:**
- No iCal `CATEGORIES` property parsing or CalDAV/Zep upstream changes.
- No user-configurable category/color mapping — the code-to-color table for absences is a fixed constant for this change, and Daylite category colors stay maintained in Daylite, not in this app.
- No changes to bare event coloring.

## Decisions

### Grouping the six codes into three hue families
- **Vacation family** (`UB` paid, `SU` special, `UU` unpaid): all fundamentally vacation-leave, sharing one hue at three intensities.
- **Sick family** (`KR`, `Kro`): both sickness-related, sharing a second hue at two intensities.
- **Time off in lieu** (`FA`): a distinct concept (compensatory time, not vacation or sickness), gets its own third hue.

This groups exactly as the user described ("similar colors for similar types") while giving `FA` genuine distinctiveness rather than forcing it into an unrelated family.

### Three new dedicated color tokens, not reused semantic tokens
Add three new theme-level custom properties in `src/app.css`, defined once (not varying per light/dark theme, since all three values were validated to sit inside both the light and dark lightness bands):

| Token | Hue family | OKLCH | Approx. hex |
|---|---|---|---|
| `--color-absence-vacation` | indigo/violet | `oklch(51.1% 0.230 277)` | `#4f46e5` |
| `--color-absence-special` | fuchsia/magenta | `oklch(59.1% 0.257 323)` | `#c026d3` |
| `--color-absence-sick` | pink/rose-magenta | `oklch(65.6% 0.212 354)` | `#ec4899` |

These were chosen and validated with the dataviz skill's `validate_palette.js` script against this app's actual light surface (`#ffffff`) and an approximation of its dark surface (`oklch(37% 0.013 285.805) ≈ #3f3f46`), alongside the existing `error` token (light `≈ #f0654a`, dark `≈ #c23b2e`) to confirm the conflict-red stays distinguishable:
- Lightness band, chroma floor, and CVD (colorblind) separation all **PASS** in both modes for the three-family set plus `error`.
- The `vacation` ↔ `special` pair sits in the CVD floor band (ΔE ≈ 12.4, just at the passing edge) and the dark-mode contrast ratios sit below the ideal 3:1 — both are acceptable under the skill's rules because every event card already shows its title as visible text (the "secondary encoding"/"relief" the skill requires when a palette only clears the floor band, not the full target).

Alternative considered: reusing `accent`/`info` for two of the three groups to avoid adding new tokens. Rejected once the light/dark hue-reassignment problem above was found — `accent` cannot be trusted to keep the same visual identity across themes, and only one free token (`info`) remains, not enough for three groups.

### Intensity mapping within a family (opacity, matching existing convention)
Reuse the codebase's existing `bg-<token>/<opacity>` convention to encode intensity within a family, rather than defining six separate hex values:
- `UB` → `bg-(--color-absence-vacation)/50`, `SU` → `bg-(--color-absence-vacation)/30`, `UU` → `bg-(--color-absence-vacation)/15`
- `KR` → `bg-(--color-absence-sick)/40`, `Kro` → `bg-(--color-absence-sick)/20`
- `FA` → `bg-(--color-absence-special)/30`
- Unmatched → `bg-info/30` (unchanged default)

### Code matching rule
Match the absence title against the six codes.
Trim the title and check for equivalence to match the title.

### Conflict indicator: status color, not a category
When a cell contains both an absence event and an assignment (`kind === "assignment"`) event for the same employee/day, add a red conflict indicator — a `ring-2 ring-error` (or equivalent border) on the cell plus a small warning icon (Lucide).
Bare (non-assignment) events also trigger the conflict indicator.

### Assignment events use the Daylite category color, status colors become the fallback
The backend fetches `GET /categories?entity=project` and keeps a category-name-to-`hex_colour` map.
Inactive categories (`is_active: false`) are still included, because existing projects can keep referencing a category after it was deactivated and should keep its color.
The calendar pipeline threads the project's category through the resolution paths: `DayliteProjectCacheEntry` gains a category field, `fetch_project_by_reference` returns the category alongside name and status, and `CalendarCellEvent` gains a nullable `category_color` carrying the resolved hex value.
Resolving the color in the backend keeps all third-party API knowledge in Rust, matching the project convention that the frontend only consumes typed Tauri responses.
Because category colors are arbitrary hex values controlled in Daylite, they cannot be expressed as static Tailwind classes; the assignment card sets its background via an inline style and picks light or dark text by relative luminance of the hex color to stay readable.
Fallback chain per assignment event: category `hex_colour` if the project has a category with a color, otherwise the existing `projectStatusToColor` mapping, otherwise the unchanged neutral placeholder color (`bg-base-300`, used when project resolution fails).

## Risks / Trade-offs

- [`vacation` ↔ `special` CVD separation sits at the floor, not the full target] → Mitigated by the event title always being visible as text on the card (secondary encoding), per the skill's floor-band rule.
- [Dark-surface contrast for the three new tokens sits below 3:1 at low opacity] → Accepted, same as the app's existing absence/status color usage (all applied as low-opacity washes with the title text as the readable channel, not the fill itself).
- [Daylite category colors are user-controlled hex values and cannot be statically validated for CVD separation against the absence and conflict colors] → Accepted, the event title stays visible as text on every card, and the conflict indicator never relies on color alone (icon plus German label).
- [Losing the status signal on assignment cards once category colors take over] → Accepted for now, the picker already restricts assignments to active statuses, so status carried almost no information in the grid.

## Migration Plan

- Add the three new CSS custom properties to `src/app.css` first (additive, no visual change yet).
- Implement and test the conflict detector in `src/app/types.ts`.
- Wire the new colors and conflict indicator into `toCellEvent()`/`timetable-cell.tsx`.
- Implement and test the Daylite categories fetch, then thread the category color through the calendar pipeline and regenerate the TypeScript bindings.
- Wire the category color into assignment rendering with the status-color fallback.
- Update the `employee-absence-display`, `assignment-persistence`, and `daylite-integration` spec deltas and archive them alongside the code change.

## Open Questions

None — codes, grouping, and conflict-highlighting behavior were confirmed by the user before writing this design. The title-matching format (leading token vs. embedded) is a documented assumption to verify during implementation, not an open design question.
