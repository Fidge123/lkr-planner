## ADDED Requirements

### Requirement: Command fixtures are generated from the real backend
The system SHALL generate frontend test fixtures by serializing each captured Tauri command's real typed output in Rust, using the same serde serialization the generated bindings are derived from.
Daylite, holiday, and CalDAV/ZEP command fixtures SHALL be captured by running the command's core logic under `VCR_MODE=replay` against committed cassettes (recorded-real).
Only local store and secret commands (for example `load_local_store`, `daylite_list_cached_contacts`, `zep_load_credentials`) SHALL be captured as type-true values built from the real Rust structs.
Fixtures SHALL be serialized deterministically (stable field order, ordered map keys, no wall-clock values) and committed under `tests/fixtures/`.

#### Scenario: HTTP-backed fixture captured from a cassette
- **WHEN** the fixture generator runs in `VCR_MODE=replay`
- **THEN** the fixtures for `daylite_list_contacts`, `get_holidays_for_week`, and `load_week_events` are produced from each command's core output against committed cassettes, with no network call

#### Scenario: Type change forces fixture regeneration
- **WHEN** a captured command's Rust type changes
- **THEN** the generator code fails to compile or produces different output until the fixture is regenerated

### Requirement: A staleness gate fails on fixture drift
The system SHALL include a `cargo test` that regenerates the fixtures in memory under replay mode and compares them to the committed files, failing if they differ.
The same generation SHALL write the committed files when `UPDATE_FIXTURES=1` is set.

#### Scenario: Drift turns the suite red
- **WHEN** a command's captured output changes and the committed fixture is not regenerated
- **THEN** `cargo test` fails and identifies the stale fixture

#### Scenario: Regeneration updates the committed fixtures
- **WHEN** the generation runs with `UPDATE_FIXTURES=1`
- **THEN** the committed `tests/fixtures/*.json` files are rewritten and the comparison test passes

### Requirement: Frontend consumes fixtures through a typed invoke mock
The system SHALL provide a test mock that replaces `@tauri-apps/api/core`'s `invoke` with one that returns the committed fixtures, so the real generated `commands` and `typedError` wrappers run against real-shaped data.
The mock SHALL throw a descriptive error for any command that has no fixture.
The fixture registry SHALL be typed against the generated bindings so a fixture not assignable to its command's success payload fails type checking.

#### Scenario: Registered command returns its fixture
- **WHEN** a component test invokes a command that has a committed fixture
- **THEN** `invoke` resolves with the fixture value and `commands`/`typedError` return it as an `ok` result

#### Scenario: Missing fixture throws
- **WHEN** the frontend invokes a command with no committed fixture
- **THEN** the mock throws an error naming the command

#### Scenario: Mismatched fixture fails type checking
- **WHEN** a fixture is not assignable to its command's generated success-payload type
- **THEN** `tsc` fails until the fixture or binding is corrected

### Requirement: Component tests run under bun test without a browser
The system SHALL run frontend component tests under `bun test` in a DOM environment (happy-dom) with React Testing Library, with no browser, dev server, or network.
At least one component test SHALL render the planning view from fixtures and assert data-driven content and an error path.

#### Scenario: Planning view renders from fixtures
- **WHEN** the planning view is rendered with the fixture-fed invoke mock installed
- **THEN** the heading and fixture-derived content are visible and no errors are thrown

#### Scenario: Error result renders the German error UI
- **WHEN** a command fixture represents an error result
- **THEN** the planning view shows the corresponding German error message

### Requirement: A minimal layout check verifies the UI renders with CSS
The system SHALL include one Playwright test on chromium that renders the app with the fixtures and asserts that the main regions are visible with non-zero size and that CSS is applied.
The test SHALL run on chromium only; cross-engine (macOS WKWebView) differences are covered by manual spot checks, not CI.

#### Scenario: App renders with applied styles
- **WHEN** the Playwright layout test renders the app on chromium with the fixture-fed `invoke` mock
- **THEN** the main view regions are visible with non-zero bounding boxes and a styled element has a non-default computed style

#### Scenario: e2e tests excluded from the bun runner
- **WHEN** `bun test` runs
- **THEN** the `*.e2e.ts` Playwright tests are not collected, because they use a suffix the native runner does not match
