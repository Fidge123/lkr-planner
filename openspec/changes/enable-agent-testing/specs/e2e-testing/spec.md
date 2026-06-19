## ADDED Requirements

### Requirement: E2E test suite runs against the Vite dev server
The system SHALL provide a `bun test:e2e` command that starts a dedicated Vite dev server (port 5174) and runs Playwright tests against it. The command SHALL exit with a non-zero code if any test fails.

#### Scenario: Successful test run
- **WHEN** `bun test:e2e` is executed in the project root
- **THEN** Playwright starts the Vite dev server, runs all tests in `tests/e2e/`, and exits 0 on success

#### Scenario: Test failure is surfaced
- **WHEN** a Playwright test assertion fails
- **THEN** the command exits with a non-zero code and prints the failing test name and assertion

### Requirement: Tauri backend calls are mocked in E2E tests
The system SHALL replace `@tauri-apps/api/core` with a test mock (`src/test/tauri-mock.ts`) when running under Playwright, via a Vite alias in `vite.playwright.config.ts`. The mock SHALL throw an error for any `invoke` call that has not been explicitly registered in a test.

#### Scenario: Registered invoke call returns stub value
- **WHEN** a Playwright test registers a mock for command `"load_local_store"` returning `{ employees: [] }`
- **THEN** any `invoke("load_local_store")` call from the frontend returns that value

#### Scenario: Unregistered invoke call throws
- **WHEN** the frontend calls `invoke` for a command that no test stub covers
- **THEN** the mock throws a descriptive error identifying the unregistered command name

### Requirement: Smoke tests cover the main application views
The system SHALL include at least one Playwright smoke test per top-level view of the application. Each smoke test SHALL verify that the view renders without JavaScript errors.

#### Scenario: Planning view loads
- **WHEN** Playwright navigates to `/`
- **THEN** the page title or a prominent heading is visible and no unhandled JavaScript errors occur

### Requirement: SessionStart hook checks the development environment
The system SHALL execute a fast environment check (`scripts/check-dev-env.ts`) at the start of each Claude Code session. The check SHALL warn (non-blocking) if any of the following are missing: `bun`, `cargo`, Playwright browser binaries.

#### Scenario: All tools present
- **WHEN** `bun`, `cargo`, and Playwright browser binaries are installed
- **THEN** the hook completes silently with exit code 0

#### Scenario: Missing Playwright browsers
- **WHEN** Playwright browser binaries have not been installed
- **THEN** the hook prints a warning message instructing the user to run `bunx playwright install` and exits 0 (non-blocking)

#### Scenario: Missing cargo
- **WHEN** `cargo` is not found on PATH
- **THEN** the hook prints a warning message and exits 0 (non-blocking)
