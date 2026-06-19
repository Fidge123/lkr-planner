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

### Mock registration must happen before navigation
The frontend calls `invoke` during initial render, so stubs registered with `page.evaluate` after `page.goto` can land too late and miss those early calls.
Tests SHALL register stubs with `page.addInitScript` so the registration runs before any application code on every navigation.
The mock exposes its registry through a stable global (`window.__tauriMock`) that the init script populates.

### Per-test mock reset
Because the Vite server process and its module graph are shared across tests, mock handler state can bleed between tests.
The mock SHALL expose a `reset()` that clears all registered handlers, and the Playwright setup SHALL call it in `beforeEach` (via an `addInitScript` that resets then registers, so each test starts from an empty registry).

### Separate `vite.playwright.config.ts`
Playwright needs to start Vite with the mock alias active, but the normal `vite.config.ts` must stay unchanged so `bun dev` and production builds are unaffected.
A thin override config extends the base config and adds the alias.

### SessionStart hook (shell script)
The hook runs a POSIX shell script (`scripts/check-dev-env.sh`) invoked directly by the shell, not through `bun`.
Running it through `bun run` would make it unable to report a missing `bun`, because the interpreter itself would be absent.
The script checks for `bun`, `cargo`, and the Playwright browser binaries on `PATH`, prints a clear non-blocking warning for anything missing, and always exits 0 to avoid slowing or blocking session startup.

## Risks / Trade-offs

- **Tauri invoke mock drift**: as `src/generated/tauri.ts` grows, tests may call commands that have no mock handler.
  - **Mitigation**: the mock throws by default for unregistered commands, requiring tests to explicitly stub each call they depend on.
- **Playwright browser download**: first run downloads roughly 100 MB of browser binaries.
  - **Mitigation**: documented in README; `scripts/check-dev-env.sh` warns if not installed.
- **Vite port conflicts**: Playwright's `webServer` uses port 5173 by default.
  - **Mitigation**: configure a dedicated port (5174) in `vite.playwright.config.ts` so it does not clash with `bun dev`.
- **Smoke test brittleness**: UI-level selectors break on refactors.
  - **Mitigation**: use `data-testid` attributes on key elements and avoid CSS class selectors.
