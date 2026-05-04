## 1. Playwright Dependencies & Configuration

- [ ] 1.1 Add `@playwright/test` as a dev dependency (`bun add -d @playwright/test`)
- [ ] 1.2 Create `vite.playwright.config.ts` that extends `vite.config.ts` and adds a `resolve.alias` mapping `@tauri-apps/api/core` to `src/test/tauri-mock.ts` and sets port to 5174
- [ ] 1.3 Create `playwright.config.ts` with `webServer` pointing to `vite.playwright.config.ts`, test directory `tests/e2e/`, and headless browser configuration
- [ ] 1.4 Add `"test:e2e": "playwright test"` script to `package.json`
- [ ] 1.5 Run `bunx playwright install chromium` to download browser binaries and verify `bun test:e2e` can start without errors

## 2. Tauri Mock Layer

- [ ] 2.1 Create `src/test/tauri-mock.ts` that exports an `invoke` function matching the `@tauri-apps/api/core` interface
- [ ] 2.2 The mock SHALL maintain a per-test handler registry (`registerMock(commandName, handler)`) exposed on `window.__tauriMock` so Playwright tests can register stubs via `page.evaluate`
- [ ] 2.3 The mock SHALL throw a descriptive error (`Unregistered Tauri command: "<name>"`) for any `invoke` call with no registered handler
- [ ] 2.4 Write a unit test (Bun) verifying the mock throws for unregistered commands and returns stub values for registered ones

## 3. Baseline Smoke Tests

- [ ] 3.1 Create `tests/e2e/smoke.spec.ts` with a test that navigates to `/` and asserts the app renders without JavaScript errors
- [ ] 3.2 Register minimal `invoke` mocks needed for the planning view to render (at minimum: `load_local_store`, `load_week_events`, `get_holidays_for_week`)
- [ ] 3.3 Add `data-testid` attributes to key structural elements in the main view component so smoke tests can use stable selectors
- [ ] 3.4 Run `bun test:e2e` and confirm all smoke tests pass

## 4. SessionStart Hook

- [ ] 4.1 Create `scripts/check-dev-env.ts` that checks for `bun`, `cargo`, and Playwright chromium binary; prints a non-blocking warning for each missing tool
- [ ] 4.2 Add a `SessionStart` hook entry to `.claude/settings.json` that runs `bun run scripts/check-dev-env.ts`
- [ ] 4.3 Verify the hook runs cleanly in a new Claude Code session and reports correctly when all tools are present
