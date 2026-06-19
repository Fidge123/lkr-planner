## 0. Prerequisites

- One-time setup (not a repeatable implementation step): run `bunx playwright install chromium` to download the browser binaries. `scripts/check-dev-env.sh` (task 4) warns when this is missing.

## 1. Playwright Dependencies & Configuration

- [ ] 1.1 (RED) Add a minimal `tests/e2e/setup.spec.ts` that navigates to `/` and asserts the document loads; running `bun test:e2e` fails because the runner, config, and script do not exist yet
- [ ] 1.2 (GREEN) Add `@playwright/test` as a dev dependency (`bun add -d @playwright/test`)
- [ ] 1.3 (GREEN) Create `vite.playwright.config.ts` that extends `vite.config.ts` and adds a `resolve.alias` mapping `@tauri-apps/api/core` to `src/test/tauri-mock.ts` and sets port to 5174
- [ ] 1.4 (GREEN) Create `playwright.config.ts` with `webServer` pointing to `vite.playwright.config.ts`, test directory `tests/e2e/`, and headless browser configuration
- [ ] 1.5 (GREEN) Add `"test:e2e": "playwright test"` script to `package.json` so `bun test:e2e` starts the dev server and the setup test passes (browser binaries from the prerequisite step must be installed)

## 2. Tauri Mock Layer

- [ ] 2.1 (RED) Write a unit test (Bun) for `src/test/tauri-mock.ts` asserting it throws for unregistered commands, returns stub values for registered ones, and clears handlers on `reset()`; it fails because the module does not exist yet
- [ ] 2.2 (GREEN) Create `src/test/tauri-mock.ts` exporting an `invoke` function matching the `@tauri-apps/api/core` interface
- [ ] 2.3 (GREEN) Maintain a handler registry exposed on `window.__tauriMock` with `registerMock(commandName, handler)` and a `reset()` that clears all handlers
- [ ] 2.4 (GREEN) Make the mock throw a descriptive error (`Unregistered Tauri command: "<name>"`) for any `invoke` call with no registered handler, so the unit test passes
- [ ] 2.5 Register stubs via `page.addInitScript` (which runs before any app code on each navigation), not `page.evaluate` after `page.goto`, so early `invoke` calls during initial render are covered
- [ ] 2.6 Add a Playwright `beforeEach` (an `addInitScript` that calls `reset()` before registering) so handler state never bleeds between tests sharing the Vite server process

## 2a. Type-safe mock registry and fixtures

- [ ] 2a.1 (RED) Add a type-level test (for example a `tsc`-checked `*.test-d.ts` or a compile-failure assertion) proving that `registerMock("load_local_store", ...)` rejects a value not assignable to the generated `LocalStore` return type; it fails because `registerMock` is currently untyped
- [ ] 2a.2 (GREEN) Make `registerMock` generic over the generated `commands` from `src/generated/tauri.ts`, so the handler's return type must match the selected command's generated return type, turning shape mismatches into `tsc` (`bun`) failures
- [ ] 2a.3 (RED) Add a test for a typed fixture builder (for example `makeLocalStore(overrides)`) asserting it returns a valid `LocalStore` and applies overrides; it fails because the builder does not exist yet
- [ ] 2a.4 (GREEN) Create reusable typed fixture builders for the commands the smoke tests depend on (`load_local_store`, `load_week_events`, `get_holidays_for_week`), typed against the generated bindings so drift surfaces in one place after regeneration

## 3. Baseline Smoke Tests

- [ ] 3.1 (RED) Create `tests/e2e/smoke.spec.ts` that navigates to `/`, registers the minimal `invoke` mocks the planning view needs using the typed fixture builders from task 2a (at minimum: `load_local_store`, `load_week_events`, `get_holidays_for_week`), and asserts a `data-testid`-selected element is visible with no unhandled JavaScript errors; it fails because the test ids do not exist yet
- [ ] 3.2 (GREEN) Add `data-testid` attributes to key structural elements in the main view component so the smoke test can use stable selectors
- [ ] 3.3 Run `bun test:e2e` and confirm all smoke tests pass

## 4. SessionStart Hook

- [ ] 4.1 (RED) Write a unit test (Bun) that runs `scripts/check-dev-env.sh` with a controlled `PATH` and asserts it warns for each missing tool (`bun`, `cargo`, Playwright chromium) and always exits 0; it fails because the script does not exist yet
- [ ] 4.2 (GREEN) Create the POSIX shell script `scripts/check-dev-env.sh` that checks for `bun`, `cargo`, and the Playwright chromium binary on `PATH`, prints a non-blocking warning for each missing tool, and always exits 0. A shell script is used (not a `bun`-run TS file) so it can still report a missing `bun`
- [ ] 4.3 Add a `SessionStart` hook entry to `.claude/settings.json` that runs `sh scripts/check-dev-env.sh` directly (not through `bun`)
- [ ] 4.4 Verify the hook runs cleanly in a new Claude Code session and reports correctly when all tools are present
