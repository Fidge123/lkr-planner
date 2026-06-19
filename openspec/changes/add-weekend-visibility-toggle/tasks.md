## 1. Persist the setting (backend)

- [ ] 1.1 (RED) Add Rust tests in `src-tauri/src/integrations/local_store.rs` asserting `DisplaySettings::default().show_weekend == false`, a load/save round-trip of `show_weekend`, and that a stored value missing the field deserializes to `false`
- [ ] 1.2 (GREEN) Add `show_weekend: bool` to `DisplaySettings` with field documentation, default it to `false`, and make missing values deserialize to `false` so the tests pass
- [ ] 1.3 Regenerate the TypeScript bindings so `DisplaySettings` includes `showWeekend: boolean`

## 2. Week-day generation

- [ ] 2.1 (RED) Extend `src/app/util.spec.ts` with failing cases asserting `getWeekDays` returns 5 days (Mon-Fri) when `showWeekend` is false and 7 days (Mon-Sun) when true
- [ ] 2.2 (GREEN) Update `getWeekDays` in `src/app/util.ts` to take a `showWeekend` flag and return 5 or 7 days so the tests pass

## 3. Display-settings service (frontend)

- [ ] 3.1 (RED) Add failing tests for `loadShowWeekend` (defaults to false when unset) and `saveShowWeekend` (persists the value while preserving `hideNonPlannableEmployees`)
- [ ] 3.2 (GREEN) Add `DEFAULT_SHOW_WEEKEND = false` and `loadShowWeekend` / `saveShowWeekend` helpers in `src/services/display-settings.ts`, merging rather than overwriting other display settings

## 4. Settings dialog toggle

- [ ] 4.1 (RED) Add a failing test that the settings dialog renders a "Wochenende anzeigen" toggle reflecting the loaded value and persists the change on save
- [ ] 4.2 (GREEN) Load `showWeekend` into the dialog state, add the German-labelled toggle with a short description under the "Anzeige" section, and save plus trigger a planning-view refresh on change

## 5. Planning view wiring

- [ ] 5.1 (RED) Add a failing test (e.g. in `src/app/page.spec.tsx`) that the planning view renders 5 day columns by default and 7 when `showWeekend` is on
- [ ] 5.2 (GREEN) Pass the loaded `showWeekend` setting into `getWeekDays` from `src/app.tsx` / `src/app/page.tsx` so the tests pass

## 6. Verification

- [ ] 6.1 Run `bun lint`, `bun test`, and `cargo test` and confirm all are green
