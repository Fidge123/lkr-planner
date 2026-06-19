## 1. Persist the setting (backend)

- [ ] 1.1 Add `show_weekend: bool` to `DisplaySettings` in `src-tauri/src/integrations/local_store.rs` with field documentation
- [ ] 1.2 Default `show_weekend` to `false` in the `Default` impl and ensure missing values deserialize to `false`
- [ ] 1.3 Add/extend Rust tests covering the default value and load/save round-trip of `show_weekend`
- [ ] 1.4 Regenerate the TypeScript bindings so `DisplaySettings` includes `showWeekend: boolean`

## 2. Display-settings service (frontend)

- [ ] 2.1 Add `DEFAULT_SHOW_WEEKEND = false` and `loadShowWeekend` / `saveShowWeekend` helpers in `src/services/display-settings.ts`
- [ ] 2.2 Preserve other display settings when saving `showWeekend` (merge, do not overwrite `hideNonPlannableEmployees`)

## 3. Week-day generation

- [ ] 3.1 Update `getWeekDays` in `src/app/util.ts` to take a `showWeekend` flag and return 5 (Mon-Fri) or 7 (Mon-Sun) days
- [ ] 3.2 Update `getWeekDays` callers (`src/app.tsx`, `src/app/page.tsx`) to pass the loaded setting
- [ ] 3.3 Update `src/app/util.spec.ts` to cover both 5-day and 7-day output

## 4. Settings dialog toggle

- [ ] 4.1 Load the current `showWeekend` value into the settings dialog state
- [ ] 4.2 Add a "Wochenende anzeigen" toggle with a short German description under the "Anzeige" section
- [ ] 4.3 Save the value and trigger a planning-view refresh on change

## 5. Verification

- [ ] 5.1 Verify the planning view shows 5 columns by default and 7 when enabled
- [ ] 5.2 Run `bun lint`, `bun test`, and `cargo test`
