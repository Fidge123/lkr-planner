## 1. Persist the setting (backend)

- [ ] 1.1 (RED) Add Rust tests in `src-tauri/src/integrations/local_store.rs` asserting `DisplaySettings::default().show_weekend == false`, a load/save round-trip of `show_weekend`, and that a stored value missing the field deserializes to `false`
- [ ] 1.2 (GREEN) Add `show_weekend: bool` to `DisplaySettings` with field documentation and `#[serde(default)]` on the field (so a stored value written before this field existed deserializes to `false`; the struct-level default does not fill individual missing fields), and set it to `false` in the existing manual `impl Default` block (do not switch to `#[derive(Default)]`, which would reset `hide_non_plannable_employees` to `false`) so the tests pass
- [ ] 1.3 (infrastructure, non-TDD) Regenerate the TypeScript bindings so `DisplaySettings` includes `showWeekend: boolean`; this is codegen output, exercised by the frontend service tests in section 3 rather than a dedicated test

## 2. Week-day generation

- [ ] 2.1 (RED) Extend `src/app/util.spec.ts` with failing cases asserting `getWeekDays` returns 5 days (Mon-Fri) when `showWeekend` is false and 7 days (Mon-Sun) when true
- [ ] 2.2 (GREEN) Update `getWeekDays` in `src/app/util.ts` to take an optional `showWeekend = false` flag and return 5 or 7 days so the tests pass; the default keeps existing callers compiling and behaving as before until they are updated in task 5.2, avoiding a `bun test` breakage window
- [ ] 2.3 (RED) Add failing cases (mocking the system date to a Saturday and a Sunday) asserting weekend-aware anchoring: with `showWeekend` on, `getWeekDays(0, true)` returns the Mon-Sun block containing today (today's weekend day is present); with `showWeekend` off, `getWeekDays(0, false)` still anchors to the upcoming Monday
- [ ] 2.4 (GREEN) Make `mondayOffset` weekend-aware in `getWeekDays`: when `showWeekend` is on use Sunday `-6` / Saturday `-5` / Monday-Friday `1 - day`; when off keep the existing Sunday `+1` / Saturday `+2` / Monday-Friday `1 - day`

## 3. Display-settings service (frontend)

- [ ] 3.1 (RED) Add a failing test proving the existing `saveHideNonPlannableEmployees` drops `showWeekend`: save `showWeekend = true`, then call `saveHideNonPlannableEmployees(false)`, and assert the reloaded store still has `showWeekend === true`
- [ ] 3.2 (GREEN) Fix `saveHideNonPlannableEmployees` to merge into the existing `displaySettings` instead of overwriting the whole object, so it no longer zeroes out `showWeekend`
- [ ] 3.3 (RED) Add failing tests for `loadShowWeekend` (defaults to false when unset) and `saveShowWeekend` (persists the value while preserving `hideNonPlannableEmployees`)
- [ ] 3.4 (GREEN) Add `DEFAULT_SHOW_WEEKEND = false` and `loadShowWeekend` / `saveShowWeekend` helpers in `src/services/display-settings.ts`, merging into the existing `displaySettings` rather than overwriting other fields

## 4. Settings dialog toggle

- [ ] 4.1 (RED) Add a failing test that the settings dialog renders a "Wochenende anzeigen" toggle reflecting the loaded value and, on save, persists the change and triggers a planning-view refresh
- [ ] 4.2 (GREEN) Load `showWeekend` into the dialog state, add the German-labelled toggle with a short description under the "Anzeige" section, and on save persist the value and trigger the planning-view refresh (matching the existing `hideNonPlannableEmployees` save-then-reload flow)

## 5. Planning view wiring

- [ ] 5.1 (RED) Add a failing test (e.g. in `src/app/page.spec.tsx`) that the planning view renders 5 day columns by default and 7 when `showWeekend` is on
- [ ] 5.2 (GREEN) Pass the loaded `showWeekend` setting into `getWeekDays` from `src/app.tsx` / `src/app/page.tsx` so the tests pass

## 6. Verification

- [ ] 6.1 Run `bun lint`, `bun test`, and `cargo test` and confirm all are green
