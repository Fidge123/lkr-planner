## ADDED Requirements

### Requirement: E2E test suite runs against the Vite dev server
The system SHALL provide a `bun test:e2e` command that starts a dedicated Vite dev server (port 5174) and runs Playwright tests against it.
The command SHALL exit with a non-zero code if any test fails.

#### Scenario: Successful test run
- **WHEN** `bun test:e2e` is executed in the project root
- **THEN** Playwright starts the Vite dev server, runs all tests in `tests/e2e/`, and exits 0 on success

#### Scenario: Test failure is surfaced
- **WHEN** a Playwright test assertion fails
- **THEN** the command exits with a non-zero code and prints the failing test name and assertion

### Requirement: Tauri backend calls are mocked in E2E tests
The system SHALL replace `@tauri-apps/api/core` with a test mock (`src/test/tauri-mock.ts`) when running under Playwright, via a Vite alias in `vite.playwright.config.ts`.
The mock SHALL throw an error for any `invoke` call that has not been explicitly registered in a test.
Tests SHALL register stubs before navigation (via `page.addInitScript`) so `invoke` calls made during initial render are covered, and the registry SHALL be reset before each test so handlers do not bleed between tests.

#### Scenario: Registered invoke call returns stub value
- **WHEN** a Playwright test registers a mock for command `"load_local_store"` returning a `LocalStore`-shaped value (for example `{ employeeSettings: [] }`, illustrative and partial) before navigating
- **THEN** any `invoke("load_local_store")` call from the frontend, including calls during initial render, returns that value

#### Scenario: Handlers reset between tests
- **WHEN** a new test starts after a previous test registered handlers
- **THEN** the mock registry is empty until the new test registers its own handlers

#### Scenario: Unregistered invoke call throws
- **WHEN** the frontend calls `invoke` for a command that no test stub covers
- **THEN** the mock throws a descriptive error identifying the unregistered command name

### Requirement: Mock stubs are type-checked against the generated bindings
The system SHALL type the mock registry and its fixtures against the generated command bindings (`src/generated/tauri.ts`), which are the source of truth derived from the Rust backend.
Because the mock replaces the raw `invoke` that the generated `typedError` wrapper consumes, the target type is the command's success payload (the value `invoke` resolves to), not the wrapped `Result` return type.
Registering a stub for a command SHALL require a return value assignable to that command's success payload, so a stub that does not match the real shape fails type checking (`bun test` / `tsc`) rather than passing silently.
Reusable typed fixture builders SHALL be provided for the commands the tests depend on, so the expected shape is defined in one place and drift surfaces as a single compile error when the Rust types change and the bindings are regenerated.

#### Scenario: Mismatched stub fails type checking
- **WHEN** a test registers a stub for `"load_local_store"` whose value is missing a field required by the generated `LocalStore` type
- **THEN** type checking fails and the test suite does not run until the stub is corrected

#### Scenario: Drift surfaces at one place after regeneration
- **WHEN** a Rust command type gains a required field and the bindings are regenerated
- **THEN** the corresponding typed fixture builder fails type checking, pointing to the single place that must be updated

### Requirement: Smoke tests cover the main application view
The system SHALL include at least one Playwright smoke test for the main view.
The application has a single top-level view (`app.tsx`, the planning view) with no router, so this main-view test is the baseline.
Each smoke test SHALL verify that the view renders without JavaScript errors, asserting on both page errors and unhandled promise rejections since unregistered commands reach the page through both channels.
The test SHALL register every command the initial render fires (`daylite_list_contacts`, `daylite_list_cached_contacts`, `load_local_store`, `load_week_events`, `get_holidays_for_week`, `zep_load_credentials`) so an unregistered command does not throw and trip the no-JS-errors assertion.

#### Scenario: Planning view loads
- **WHEN** Playwright registers the startup command mocks and navigates to `/`
- **THEN** the main view (`data-testid="planning-view"`) and the heading are visible, and no page errors or unhandled rejections occur

### Requirement: SessionStart hook checks the development environment
The system SHALL execute a fast environment check at the start of each Claude Code session via a POSIX shell script (`scripts/check-dev-env.sh`) run directly by the shell, not through `bun`, so that a missing `bun` can still be reported.
The check SHALL warn (non-blocking) if any of the following are missing: `bun`, `cargo`, Playwright browser binaries.

#### Scenario: All tools present
- **WHEN** `bun`, `cargo`, and Playwright browser binaries are installed
- **THEN** the hook completes silently with exit code 0

#### Scenario: Missing Playwright browsers
- **WHEN** Playwright browser binaries have not been installed
- **THEN** the hook prints a warning message instructing the user to run `bunx playwright install` and exits 0 (non-blocking)

#### Scenario: Missing cargo
- **WHEN** `cargo` is not found on PATH
- **THEN** the hook prints a warning message and exits 0 (non-blocking)

#### Scenario: Missing bun
- **WHEN** `bun` is not found on PATH
- **THEN** the shell script still runs and prints a warning identifying `bun` as missing, and exits 0 (non-blocking)
