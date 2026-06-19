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
**Decision**: Add `show_weekend: bool` to the Rust `DisplaySettings` struct, defaulting to `false`, and mirror it as `showWeekend: boolean` in the generated TypeScript type.
- `DisplaySettings` has a hand-written `impl Default` (not `#[derive(Default)]`) because `hide_non_plannable_employees` defaults to `true`.
- The new field must be added to that manual block as `show_weekend: false`; switching to `#[derive(Default)]` would silently reset `hide_non_plannable_employees` to `false`.
- The `show_weekend` field itself must carry `#[serde(default)]` so a stored `DisplaySettings` written before this field existed deserializes (the struct-level default does not fill in individual missing fields), resolving to `false`.
- Keeps a single display-settings object rather than introducing a new persistence surface.

### Week-day generation
**Decision**: Change `getWeekDays` to accept the weekend flag and return 5 or 7 days.
- Signature becomes `getWeekDays(weekOffset, showWeekend)`; length is `showWeekend ? 7 : 5`.
- Saturday and Sunday are appended after Friday, keeping Monday as the first column.
- Downstream components need no structural change because they already key off `weekDays.length`.

### Frontend wiring
**Decision**: Load `showWeekend` alongside the existing display settings in `app.tsx`, pass it into `getWeekDays`, and extend `display-settings.ts` with load/save helpers plus a `DEFAULT_SHOW_WEEKEND = false` constant.
- The settings dialog adds a toggle mirroring the `hideNonPlannable` checkbox, labelled "Wochenende anzeigen" with a short German description.

### Fix the existing overwrite bug in `saveHideNonPlannableEmployees`
**Decision**: Change both save helpers to merge into the existing `displaySettings` rather than replacing it.
- Today `saveHideNonPlannableEmployees` writes `displaySettings: { hideNonPlannableEmployees }`, replacing the whole object.
- With a second field live, the helper that saves last wins and silently drops the other field's value.
- `saveHideNonPlannableEmployees` and the new `saveShowWeekend` must both spread the loaded `displaySettings` and override only their own field.

## Risks / Trade-offs

- **Risk**: Existing local stores lack the new field.
  - **Mitigation**: serde default and a frontend default of `false` resolve missing values to off.
- **Risk**: The existing `saveHideNonPlannableEmployees` overwrites the whole `displaySettings` object, so adding a second field means one save can drop the other.
  - **Mitigation**: fix both save helpers to merge into the loaded `displaySettings`, covered by a dedicated regression test.
- **Trade-off**: `getWeekDays` gains a parameter, touching its callers and tests.
  - Accepted because it keeps the day list as the single source of truth for column count.
- **Risk**: Weekend columns may surface holidays or absences not previously visible.
  - **Mitigation**: those item types already render by date, so showing them on Saturday/Sunday is the intended behavior, not a regression.
