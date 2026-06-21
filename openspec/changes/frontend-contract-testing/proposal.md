## Why

Coding agents generate PRs that we want to merge with confidence without extensive manual testing.
The existing suites (Bun unit tests, Cargo unit and VCR tests) leave one gap unguarded: nothing checks that the React frontend correctly consumes what the Rust backend actually returns.
Hand-written `invoke` mocks (the approach explored in `enable-agent-testing`) can silently drift from the real command shapes, so a backend change an agent forgets to reflect in the frontend still passes green.
We need fast, stable frontend tests fed by fixtures generated from the real backend, with a staleness gate so that a backend shape change the frontend mishandles turns CI red.

## What Changes

- Replace the Playwright browser approach (`enable-agent-testing`) with component tests in `bun test` using a DOM environment (happy-dom) and React Testing Library, so the frontend layer is fast, deterministic, and needs no browser or dev server.
- Add a fixture-capture step in Rust that serializes each relevant Tauri command's real typed output to committed JSON fixtures: Daylite commands are captured from the existing cassettes under `VCR_MODE=replay`, and store, secret, holiday, and CalDAV commands are captured as type-true seeded values built from the real Rust structs.
- Add a staleness gate inside `cargo test`: the fixtures are regenerated in memory and compared to the committed files, so a command whose shape or value changes fails the test until the fixtures are regenerated.
- Provide a typed `invoke` mock that feeds the fixtures to the frontend, typed against the generated bindings, so the real `commands` and `typedError` wrappers execute against real-shaped data.
- Add baseline component tests for the planning view that consume the fixtures and assert rendered content, interactions, and error states.

## Capabilities

### New Capabilities

- `frontend-contract-testing`: generated-fixture-driven frontend component tests with a backend staleness gate, giving high-confidence coverage that the frontend correctly consumes real backend command output.

### Removed Capabilities

- Supersedes the `enable-agent-testing` change (Playwright browser smoke with hand-authored mocks), which is removed in favor of this approach.

## Impact

- New: a Rust fixture-capture test plus a staleness-gate test, `tests/fixtures/` JSON files, a frontend DOM test setup (happy-dom preload plus React Testing Library), a typed fixture-fed `invoke` mock, and planning-view component tests.
- `package.json`: new dev dependencies (`@happy-dom/global-registrator`, `@testing-library/react`, `@testing-library/dom`), a `bunfig.toml` test preload, and a `fixtures:generate` script.
- CI: the staleness gate runs inside the existing `cargo test` (rust job already unlocks cassettes); component tests run inside the existing `bun` job. No browser job is added.
- Removes the `enable-agent-testing` change proposal; no Playwright tooling is introduced.

## Out of Scope

- Real-application end-to-end tests through the built Tauri app and WKWebView (IPC and serialization layer); avoided to keep CI fast and free of browser/WebDriver flakiness.
- Extending the record/replay seam to the CalDAV/ZEP and holiday clients; those commands ship type-true seeded fixtures for now and can become recorded-real in a later change.
- Visual regression and real layout/CSS fidelity; covered by manual spot checks on macOS.
