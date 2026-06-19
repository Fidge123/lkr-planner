## Context

The planning view builds its columns from `getWeekDays(weekOffset)` in `src/app/util.ts`, which currently hardcodes `length: 5` to return Monday to Friday.
The table and child components already derive their column count from `weekDays.length` (for example `colSpan={weekDays.length + 1}`), so they adapt to a variable number of days.

Display preferences are already persisted via `DisplaySettings` in the local store (`src-tauri/src/integrations/local_store.rs`), exposed to the frontend through the generated `DisplaySettings` type and the `display-settings.ts` service, and toggled in the settings dialog under the "Anzeige" section (`hideNonPlannableEmployees` is the existing precedent).

## Goals / Non-Goals

**Goals:**
- Add a persisted `showWeekend` display setting, defaulting to off.
- Make `getWeekDays` return Monday to Friday or Monday to Sunday based on the setting.
- Surface the setting as a German-labelled toggle in the settings dialog, reusing the existing display-settings pattern.

**Non-Goals:**
- Per-user or per-employee weekend preferences.
- Hiding individual weekend days independently (Saturday only, etc.).
- Any change to weekend assignment data, holidays, or absences beyond rendering the extra columns.

## Decisions

### Storage: extend DisplaySettings
**Decision**: Add `show_weekend: bool` to the Rust `DisplaySettings` struct with a `Default` of `false`, and mirror it as `showWeekend: boolean` in the generated TypeScript type.
- Follows the existing `hide_non_plannable_employees` precedent exactly, including `#[serde(default)]`-style tolerance so older local stores without the field load as off.
- Keeps a single display-settings object rather than introducing a new persistence surface.

### Week-day generation
**Decision**: Change `getWeekDays` to accept the weekend flag and return 5 or 7 days.
- Signature becomes `getWeekDays(weekOffset, showWeekend)`; length is `showWeekend ? 7 : 5`.
- Saturday and Sunday are appended after Friday, keeping Monday as the first column.
- Downstream components need no structural change because they already key off `weekDays.length`.

### Frontend wiring
**Decision**: Load `showWeekend` alongside the existing display settings in `app.tsx`, pass it into `getWeekDays`, and extend `display-settings.ts` with load/save helpers plus a `DEFAULT_SHOW_WEEKEND = false` constant.
- The settings dialog adds a toggle mirroring the `hideNonPlannable` checkbox, labelled "Wochenende anzeigen" with a short German description.

## Risks / Trade-offs

- **Risk**: Existing local stores lack the new field. **Mitigation**: serde default and a frontend default of `false` resolve missing values to off.
- **Trade-off**: `getWeekDays` gains a parameter, touching its callers and tests. Accepted because it keeps the day list as the single source of truth for column count.
- **Risk**: Weekend columns may surface holidays or absences not previously visible. **Mitigation**: those item types already render by date, so showing them on Saturday/Sunday is the intended behavior, not a regression.
