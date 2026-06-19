## Context

The app is a Tauri v2 desktop application with a React/TypeScript frontend (Vite) and a Rust backend.
All external API calls go through Tauri commands (`invoke`).
The generated bindings live in `src/generated/tauri.ts` and wrap `@tauri-apps/api/core`'s `invoke`.
Existing test suites are unit-only (Bun for TS, Cargo for Rust).
There is no way to run and observe the full UI today.

The Stop hook already runs `bun lint && bun test && cargo test`, so Claude gets test feedback after each session.
The gap is UI-level feedback: Claude cannot start the app, navigate it, and assert visual or interactive behavior.

## Goals / Non-Goals

**Goals:**
- Claude can run `bun test:e2e` to exercise the frontend in a real browser
- Playwright starts and stops the Vite dev server automatically
- `@tauri-apps/api/core` is replaced with a controllable mock at test time via a Vite alias
- A `SessionStart` hook verifies the environment (deps installed, Rust toolchain present) so Claude gets early failure rather than silent missing-tool errors
- Baseline smoke tests cover the app's main views so regressions are visible

**Non-Goals:**
- Testing the Rust backend through Playwright (Cargo tests cover that)
- Running the full Tauri desktop shell in tests (unnecessary complexity, Vite-only is sufficient for UI tests)
- Visual regression / screenshot diffing (out of scope for now)
- CI integration (purely local agent tooling)
- Adding `test:e2e` to the `Stop` hook: Playwright needs a browser and is too slow for the post-session loop, so it stays run-on-demand and the `Stop` hook keeps running only `bun lint && bun test && cargo test`
- Per-modal smoke tests: the app has a single view (`app.tsx`) with two modals (`SettingsDialog`, `EmployeeIcalDialog`) and no router, so one main-view smoke test is the baseline and the modals are not separately covered

## Decisions

### Playwright over Cypress
Playwright has first-class Vite integration via `webServer`, native TypeScript support without extra setup, and runs fully headless.
Cypress requires a separate server process and has historically had flakier Vite/ESM support.

### Tauri mock via Vite alias, not `vi.mock`
At build time (Playwright uses the running Vite server, not Vitest), the cleanest intercept point is a Vite `resolve.alias` in a dedicated Playwright-specific Vite config.
This replaces `@tauri-apps/api/core` with `src/test/tauri-mock.ts`, which exports an `invoke` function matching the real signature.
Tests control return values per command name.

Alternative considered: patching `window.__TAURI_INTERNALS` at runtime via Playwright's `page.addInitScript`.
This works but is less type-safe and harder to reset between tests.

### The startup mock set is the six commands the initial render fires
Running the smoke test is what pins this set down; a static read of `app.tsx` alone undercounts it.
The initial render fires six commands across the always-mounted component tree: `daylite_list_contacts` (Daylite sync in `app.tsx`), `daylite_list_cached_contacts` (the planning employees hook), `load_local_store`, `load_week_events` (assignments reload), `get_holidays_for_week` (holidays effect), and `zep_load_credentials` (the `SettingsDialog`, which is always mounted even while closed).
Because the mock throws by default for unregistered commands, the smoke test SHALL register all six.
Some callers catch (`daylite_list_contacts`, `load_week_events`) and some do not (`load_local_store` is uncaught; `get_holidays_for_week` and `zep_load_credentials` use `.then` without `.catch`; `daylite_list_cached_contacts` propagates), so leaving any unregistered surfaces as a page error or unhandled rejection that trips the no-JS-errors assertion.
React StrictMode (in `main.tsx`) double-invokes mount effects in dev, so a missing handler can surface twice.
The smoke test therefore asserts on both `pageerror` events and `unhandledrejection`, since unregistered commands reach the page through both channels.

### Mock registration must happen before navigation
The frontend calls `invoke` during initial render, so stubs registered with `page.evaluate` after `page.goto` can land too late and miss those early calls.
Tests SHALL register stubs with `page.addInitScript` so the registration runs before any application code on every navigation.
The mock exposes its registry through a stable global (`window.__tauriMock`) that the init script populates.

### Per-test mock reset
Because the Vite server process and its module graph are shared across tests, mock handler state can bleed between tests.
The mock SHALL expose a `reset()` that clears all registered handlers, and the Playwright setup SHALL call it in `beforeEach` (via an `addInitScript` that resets then registers, so each test starts from an empty registry).

### Type-safe mocks against the generated bindings
The mock data is hand-authored per test, not recorded from a real backend, so without a type boundary it can silently drift from the real shapes.
`src/generated/tauri.ts` (produced by tauri-specta from the Rust structs and commands) is the source of truth for command argument and return types.
The mock registry SHALL be generic over the generated `commands`, so `registerMock(name, value)` only accepts a value assignable to that command's success payload; a mismatched stub then fails `tsc` instead of passing silently.
Type checking is wired in as a gate via `"test:e2e": "tsc --noEmit && playwright test"` (and a `typecheck` script), and `tsconfig.json` includes `tests` so the registration call sites are checked too, so a bad stub stops the E2E run before the browser starts.
The target type is the success payload, not the command's wrapped `Result` return type.
The mock replaces the raw `invoke`, and the generated binding wraps that call in `typedError`, so `commands.loadLocalStore` reads `typedError<LocalStore, StoreError>(invoke("load_local_store"))`.
The handler therefore returns the raw `LocalStore` that `invoke` resolves to, recovered from the bindings as `Extract<Awaited<ReturnType<typeof commands[K]>>, { status: "ok" }>` with a conditional `infer` to read its `data` (indexing `["data"]` directly is rejected for a generic type parameter).
The generated file carries `// @ts-nocheck`, but that only suppresses errors inside the file; its exported inferred types still flow to consumers, so this derivation works.
Dispatch and typing key on different strings: the frontend passes the snake_case command name to `invoke` (`"load_local_store"`), while the `commands` object keys on camelCase (`loadLocalStore`).
A `CamelToSnake` template-literal type derives the snake_case names from the bindings automatically, so the registry bridges the two with no hand-maintained name table and new commands need no extra wiring.
Reusable typed fixture builders (for example `makeLocalStore(overrides)`) SHALL define each command's expected shape in one place against the generated types, so tests compose fixtures instead of scattering object literals, and a Rust type change plus regeneration surfaces as a single compile error in the builder.
Note the `page.addInitScript` boundary serializes the handler to a string, so the typed `registerMock` wrapper and fixtures live in Node-side test code; only the already-constructed, type-checked data crosses into the browser.
Error-path fixtures are out of scope for the smoke tests, but note for later that `typedError` rethrows real `Error` instances and only converts non-`Error` rejections into a `{ status: "error" }` result, so simulating a backend error requires rejecting the mock with a non-`Error` value.

### Separate `vite.playwright.config.ts`
Playwright needs to start Vite with the mock alias active, but the normal `vite.config.ts` must stay unchanged so `bun dev` and production builds are unaffected.
A thin override config extends the base config (via `mergeConfig`) and adds the alias.

### E2E files use a `.e2e.ts` suffix to stay out of the native `bun test` runner
The `Stop` hook runs `bun test`, the native runner, which scans the whole repo for `*.spec.ts` / `*.test.ts` and would collect the Playwright specs, then crash because `@playwright/test`'s `test()` cannot run under the Bun runner.
Bun has no path-exclude for test discovery, and a single `bunfig` `root` cannot cover both `src` and `scripts`, so the robust split is by filename: E2E tests are named `*.e2e.ts` and `playwright.config.ts` sets `testMatch: "**/*.e2e.ts"`.
The two runners then never collect each other's files.
This is a deviation from the original task filenames (`setup.spec.ts`, `smoke.spec.ts`), made to keep both `bun test` and `bun test:e2e` green.

### Playwright pinned to 1.56.1 for the web environment
Claude Code on the web pre-stages the Playwright browser at `/opt/pw-browsers` for a specific Playwright version (chromium 141, revision 1194, matching `@playwright/test` 1.56.1).
A floating `^` would resolve to a newer Playwright that expects a different browser revision and fail to find the pre-staged one, with no network to download it.
Pinning to `1.56.1` lets the E2E suite run in web sessions without a browser download; local machines still run `bunx playwright install` once.

### SessionStart hook (shell script)
The hook runs a POSIX shell script (`scripts/check-dev-env.sh`) invoked directly by the shell, not through `bun`.
Running it through `bun run` would make it unable to report a missing `bun`, because the interpreter itself would be absent.
The script checks for `bun`, `cargo`, and the Playwright browser binaries on `PATH`, prints a clear non-blocking warning for anything missing, and always exits 0 to avoid slowing or blocking session startup.

## Risks / Trade-offs

- **Tauri invoke mock drift**: as `src/generated/tauri.ts` grows, tests may call commands that have no mock handler, or stub a command with a value that no longer matches its real shape.
  - **Mitigation**: the mock throws by default for unregistered commands (catching missing handlers at runtime), and the registry plus fixtures are typed against the generated bindings (catching shape mismatches at compile time), so both the call coverage and the data shape are guarded.
- **Playwright browser download**: first run downloads roughly 100 MB of browser binaries.
  - **Mitigation**: captured as the one-time prerequisite step in tasks.md, and `scripts/check-dev-env.sh` warns if not installed.
- **Vite port conflicts**: the normal dev server runs on port 1420 (`strictPort: true` in `vite.config.ts`), and `bun dev` will fail to start if that port is taken.
  - **Mitigation**: configure a dedicated port (5174) in `vite.playwright.config.ts` so a running Playwright server never occupies 1420 and `bun dev` stays unaffected.
- **Smoke test brittleness**: UI-level selectors break on refactors.
  - **Mitigation**: use `data-testid` attributes on key elements and avoid CSS class selectors.
