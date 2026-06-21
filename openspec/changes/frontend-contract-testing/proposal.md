## Why

Coding agents generate PRs that we want to merge with confidence without extensive manual testing.
The existing suites (Bun unit tests, Cargo unit and VCR tests) leave one gap unguarded: nothing checks that the React frontend correctly consumes what the Rust backend actually returns.
Hand-written `invoke` mocks (the approach explored in `enable-agent-testing`) can silently drift from the real command shapes, so a backend change an agent forgets to reflect in the frontend still passes green.
We need fast, stable frontend tests fed by fixtures generated from the real backend, with a staleness gate so that a backend shape change the frontend mishandles turns CI red, plus a thin layout check so an agent can see that the UI actually renders with its CSS.

## What Changes

- Generalize the existing `#[cfg(test)]` record/replay transport (today Daylite-only) into a reusable HTTP transport so the holidays (Nager) and CalDAV/ZEP clients can also record and replay cassettes.
- Add recording harnesses for holidays and ZEP/CalDAV (live, `#[ignore]`, mirroring the Daylite harness) and commit the resulting cassettes, so those commands' fixtures are recorded-real rather than seeded.
- Add a fixture-capture step in Rust that serializes each planning-view command's real typed output to committed JSON fixtures: Daylite, holidays, and CalDAV from cassettes under `VCR_MODE=replay`; the remaining local store/secret commands as type-true seeded values built from the real Rust structs.
- Add a staleness gate inside `cargo test`: fixtures are regenerated in memory and compared to the committed files, so a command whose shape or value changes fails the test until the fixtures are regenerated.
- Add component tests in `bun test` using a DOM environment (happy-dom) and React Testing Library, fed the fixtures through a typed `invoke` mock so the real `commands`/`typedError` wrappers run against real-shaped data.
- Add one minimal Playwright test (chromium) that renders the app and asserts basic layout and that CSS is applied, so an agent can confirm the UI is not visually broken.

## Capabilities

### New Capabilities

- `frontend-contract-testing`: generated-fixture-driven frontend component tests with a backend staleness gate, plus a minimal layout check, giving high-confidence coverage that the frontend correctly consumes real backend output and renders.

### Modified Capabilities

- `http-recording`: the record/replay seam is generalized from Daylite-only to a shared HTTP transport that holidays and CalDAV/ZEP also use.

### Removed Capabilities

- Supersedes the `enable-agent-testing` change (Playwright browser smoke with hand-authored mocks), which is removed in favor of this approach.

## Impact

- Backend: a shared record/replay HTTP transport (extracted from the Daylite client), record/replay seams in the holidays and CalDAV/ZEP clients, recording harnesses, and new committed cassettes for holidays and ZEP/CalDAV.
- New test assets: a Rust fixture-capture plus staleness-gate test, `tests/fixtures/*.json`, a frontend DOM test setup (happy-dom preload plus React Testing Library), a typed fixture-fed `invoke` mock, planning-view component tests, and one chromium Playwright layout test.
- `package.json`: new dev dependencies (`@happy-dom/global-registrator`, `@testing-library/react`, `@testing-library/dom`, `@playwright/test`), a `bunfig.toml` test preload, and `fixtures:generate` / `test:e2e` scripts.
- CI: the staleness gate and recorded replay run inside the existing `cargo test` (rust job already unlocks cassettes); component tests run inside the existing `bun` job; one small chromium-only Playwright job is added for the layout check.
- Removes the `enable-agent-testing` change proposal.

## Prerequisites

- Recording the ZEP/CalDAV cassette requires live ZEP credentials and a reachable CalDAV server (a manual `VCR_MODE=record` run, like the Daylite harness). The seam, replay capture, and tests do not depend on live access; only the one-time recording does.

## Out of Scope

- Real-application end-to-end tests through the built Tauri app and WKWebView (IPC and serialization layer); avoided to keep CI fast and free of WebDriver flakiness.
- Cross-browser-engine layout verification in CI; the layout check runs on chromium only, and macOS WKWebView differences are covered by manual spot checks.
- Visual regression / screenshot diffing.
