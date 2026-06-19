## 0. Prerequisites

- One-time setup (not a repeatable implementation step): run `bunx playwright install chromium` to download the browser binaries. `scripts/check-dev-env.sh` (task 4) warns when this is missing. On Claude Code on the web the browser is pre-staged at `/opt/pw-browsers` (matching `@playwright/test`, pinned to 1.56.1), so no download is needed there.

## 1. Playwright Dependencies & Configuration

- [x] 1.1 (RED) Add a minimal `tests/e2e/setup.e2e.ts` that navigates to `/` and asserts the document loads; running `bun test:e2e` fails because the runner, config, and script do not exist yet. The `.e2e.ts` suffix (not `.spec.ts`) keeps the file out of the native `bun test` runner, which scans the whole repo for `*.spec.ts` and would otherwise try to execute Playwright tests
- [x] 1.2 (GREEN) Add `@playwright/test` as a dev dependency (`bun add -d @playwright/test`), pinned to `1.56.1` to match the browser pre-staged in the web environment
- [x] 1.3 (GREEN) Create `vite.playwright.config.ts` that extends `vite.config.ts` (via `mergeConfig`) and adds a `resolve.alias` mapping `@tauri-apps/api/core` to `src/test/tauri-mock.ts` and sets port to 5174
- [x] 1.4 (GREEN) Create `playwright.config.ts` with `webServer` pointing to `vite.playwright.config.ts`, test directory `tests/e2e/`, `testMatch: "**/*.e2e.ts"`, and a headless chromium project
- [x] 1.5 (GREEN) Add `"test:e2e": "tsc --noEmit && playwright test"` script to `package.json` so `bun test:e2e` type-checks the mocks and fixtures, then starts the dev server and the setup test passes (browser binaries from the prerequisite step must be installed)

## 2. Tauri Mock Layer

- [x] 2.1 (RED) Write a unit test (Bun) for `src/test/tauri-mock.ts` asserting it throws for unregistered commands, returns stub values for registered ones, and clears handlers on `reset()`; it fails because the module does not exist yet
- [x] 2.2 (GREEN) Create `src/test/tauri-mock.ts` exporting an `invoke` function matching the `@tauri-apps/api/core` interface
- [x] 2.3 (GREEN) Maintain a handler registry exposed on `window.__tauriMock` with `registerMock(commandName, handler)` and a `reset()` that clears all handlers. The registry is read lazily on every `invoke`, so an init script that installs `window.__tauriMock` after the module loads is still picked up
- [x] 2.4 (GREEN) Make the mock throw a descriptive error (`Unregistered Tauri command: "<name>"`) for any `invoke` call with no registered handler, so the unit test passes
- [x] 2.5 Register stubs via `page.addInitScript` (which runs before any app code on each navigation), not `page.evaluate` after `page.goto`, so early `invoke` calls during initial render are covered. Implemented in `tests/e2e/support/tauri-mock-page.ts`
- [x] 2.6 The init script rebuilds the registry from scratch on every navigation (an implicit reset before registering), so handler state never bleeds between tests sharing the Vite server process

## 2a. Type-safe mock registry and fixtures

- [x] 2a.1 (RED) Add a type-level test (`src/test/tauri-mock-fixtures.test-d.ts`, checked by `tsc --noEmit`) proving that `registerMock("load_local_store", ...)` rejects a value not assignable to the generated `LocalStore` payload; it fails because `registerMock` is currently untyped
- [x] 2a.2 (GREEN) Make `registerMock` generic over the generated `commands` from `src/generated/tauri.ts`, so the handler's return type must match the selected command's success payload (the raw value `invoke` resolves to, which `typedError` then wraps), not the wrapped `Result` return type. Derived via `Extract<Awaited<ReturnType<...>>, { status: "ok" }>` with a conditional `infer` (indexing `["data"]` directly is rejected for a generic). `// @ts-nocheck` on the generated file does not block this: its exported inferred types still flow to consumers
- [x] 2a.2a (GREEN) Bridge the key mismatch: runtime dispatch keys on the snake_case command string (`"load_local_store"`) that the frontend passes to `invoke`, while the generated `commands` object keys on camelCase (`loadLocalStore`). A `CamelToSnake` template-literal type derives the snake names from the bindings automatically, so the typed registry maps between the two with no hand-maintained table
- [x] 2a.3 (RED) Add a test for a typed fixture builder (`makeLocalStore(overrides)`) asserting it returns a valid `LocalStore` and applies overrides; it fails because the builder does not exist yet
- [x] 2a.4 (GREEN) Create reusable typed fixture builders for the commands the smoke tests depend on (`makeContacts` for both `daylite_list_contacts` and `daylite_list_cached_contacts`, `makeLocalStore`, `makeWeekEvents`, `makeHolidays`), each returning the command's success payload typed against the generated bindings so drift surfaces in one place after regeneration. `zep_load_credentials` returns `null` and needs no builder

## 3. Baseline Smoke Tests

- [x] 3.1 (RED) Create `tests/e2e/smoke.e2e.ts` that navigates to `/`, registers the `invoke` mocks the planning view fires during initial render using the typed fixture builders from task 2a, and asserts a `data-testid`-selected element is visible with no page errors and no unhandled rejections; it fails because the test ids do not exist yet. The startup set discovered by running the test is six commands: `daylite_list_contacts` (Daylite sync), `daylite_list_cached_contacts` (planning employees hook), `load_local_store`, `load_week_events`, `get_holidays_for_week`, and `zep_load_credentials` (the always-mounted settings dialog). React StrictMode double-invokes mount effects in dev, so an unregistered command can surface twice; some callers catch and some do not, so all six must be registered
- [x] 3.2 (GREEN) Add a `data-testid="planning-view"` attribute to the main view root in `src/app.tsx` so the smoke test can use a stable selector
- [x] 3.3 Run `bun test:e2e` and confirm all smoke tests pass (2 passed, exit 0, against the pre-staged chromium)

## 4. SessionStart Hook

- [x] 4.1 (RED) Write a unit test (Bun) that runs `scripts/check-dev-env.sh` with a controlled `PATH` and asserts it warns for each missing tool (`bun`, `cargo`, Playwright chromium) and always exits 0, plus a case proving it stays silent when all are present; it fails because the script does not exist yet
- [x] 4.2 (GREEN) Create the POSIX shell script `scripts/check-dev-env.sh` that checks for `bun`, `cargo`, and the Playwright browsers chromium and webkit (each engine's `INSTALLATION_COMPLETE` marker under `PLAYWRIGHT_BROWSERS_PATH` or the default cache), prints a non-blocking warning for each missing tool, and always exits 0. It uses only shell builtins so it works with an empty `PATH`, and is a shell script (not a `bun`-run TS file) so it can still report a missing `bun`
- [x] 4.3 Add a `SessionStart` hook entry to `.claude/settings.json` that runs `sh scripts/check-dev-env.sh` directly (not through `bun`)
- [x] 4.4 Verify the hook command runs cleanly: with `bun` and `cargo` present and chromium pre-staged it exits 0 silently. A full new-session run is exercised when this branch is next opened on the web
