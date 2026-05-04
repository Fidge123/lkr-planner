## Why

Claude currently cannot verify UI and integration behavior by running the application, which limits code quality when using agentic engineering. Adding Playwright-based end-to-end testing against the Vite frontend — with mocked Tauri backend calls — gives Claude a reliable way to start the app, interact with it, and assert correctness before declaring tasks complete.

## What Changes

- Add Playwright as an E2E testing framework targeting the Vite dev server
- Add a Tauri API mock layer so `invoke()` calls work in a browser test context
- Add `test:e2e` script to `package.json`
- Add a `SessionStart` hook so Claude automatically verifies the dev environment is ready at the start of each session
- Provide a small set of smoke tests covering the app's main views as a baseline

## Capabilities

### New Capabilities

- `e2e-testing`: Playwright setup with Vite dev server and mocked Tauri backend, plus a `test:e2e` script and baseline smoke tests, enabling Claude to run automated UI tests

### Modified Capabilities

- `http-recording`: No requirement changes — implementation detail only (Playwright test runner is separate from VCR tests)

## Impact

- `package.json`: new `test:e2e` script, new Playwright devDependencies
- `.claude/settings.json`: new `SessionStart` hook
- New files: `playwright.config.ts`, `src/test/tauri-mock.ts`, `tests/e2e/` directory with baseline smoke tests
- No changes to Rust backend or existing Bun/Cargo test suites
