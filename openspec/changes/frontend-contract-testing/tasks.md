## 1. Generalize the record/replay transport

- [ ] 1.1 (RED) Add a Cargo test asserting a shared record/replay HTTP transport replays a committed cassette without a network call and records sanitized interactions; it fails because the transport is still Daylite-private
- [ ] 1.2 (GREEN) Extract the `#[cfg(test)]` record/replay transport from the Daylite client into a shared module keyed on method/path/query/body, reusing `http_record_replay.rs`
- [ ] 1.3 (GREEN) Route the Daylite client through the shared transport and keep all existing Daylite cassette tests green (regression guard)

## 2. Record holidays and ZEP/CalDAV cassettes

- [ ] 2.1 (RED) Add a replay test for `get_holidays_for_week`'s fetch that expects a committed `holidays-*.json` cassette; it fails until the seam and cassette exist
- [ ] 2.2 (GREEN) Route the holidays (Nager) client through the shared transport and add an `#[ignore]` recording harness; record and commit the holidays cassette (public API, no credentials)
- [ ] 2.3 (RED) Add a replay test for `load_week_events`'s CalDAV fetch that expects a committed `zep-week-events-*.json` cassette; it fails until the seam and cassette exist
- [ ] 2.4 (GREEN) Route the CalDAV/ZEP client through the shared transport and add an `#[ignore]` recording harness (Basic auth, REPORT/PROPFIND)
- [ ] 2.5 Record and commit the ZEP/CalDAV cassette via a one-time `VCR_MODE=record` run with live credentials (manual prerequisite), and confirm the replay tests pass

## 3. Fixture capture and staleness gate (Rust)

- [ ] 3.1 (RED) Add a Cargo test `command_fixtures_up_to_date` that regenerates the planning-view fixtures under `VCR_MODE=replay` and compares them to `tests/fixtures/*.json`; it fails because the generator and fixtures do not exist yet
- [ ] 3.2 (GREEN) Capture recorded-real fixtures for `daylite_list_contacts`, `get_holidays_for_week`, and `load_week_events` from cassettes, and type-true seeded fixtures for `load_local_store` and `daylite_list_cached_contacts`, serializing each command's success payload
- [ ] 3.3 (GREEN) Serialize deterministically (stable field order, ordered map keys, fixed seed dates); write files when `UPDATE_FIXTURES=1`, generate the committed fixtures, add a `fixtures:generate` script, and make the comparison test pass
- [ ] 3.4 Confirm the gate fails on drift: changing a captured value without regenerating makes `cargo test` red

## 4. Frontend DOM test environment and invoke mock

- [ ] 4.1 (RED) Add a component test that renders a trivial element with React Testing Library; it fails because no DOM environment is configured
- [ ] 4.2 (GREEN) Add dev deps `@happy-dom/global-registrator`, `@testing-library/react`, `@testing-library/dom`, and a `bunfig.toml` `[test] preload` that registers happy-dom and RTL cleanup, so the test passes
- [ ] 4.3 (RED) Add a unit test asserting the fixture mock resolves `invoke("load_local_store")` to the committed fixture and throws for an unmapped command; it fails because the mock does not exist
- [ ] 4.4 (GREEN) Create `src/test/tauri-fixture-mock.ts` that loads `tests/fixtures/*.json`, exposes `invoke(command, args?)` returning the fixture (throwing for unmapped commands) installable via `mock.module("@tauri-apps/api/core", ...)`, typed against the generated bindings (success-payload per command)

## 5. Component tests for the planning view

- [ ] 5.1 (RED) Add `src/app.contract.spec.tsx` that installs the fixture mock, renders `<App />`, and asserts the planning view renders fixture-derived content with no errors; it fails before the mock and fixtures are wired
- [ ] 5.2 (GREEN) Wire the fixtures so the planning view renders, asserting at least one data-driven value from a fixture (for example a contact or holiday) appears
- [ ] 5.3 (GREEN) Add an error-path test: a command fixture representing an error result renders the German error UI

## 6. Minimal Playwright layout check

- [ ] 6.1 (RED) Add `tests/e2e/layout.e2e.ts` that renders the app and asserts the main regions are visible with non-zero size and that CSS is applied (a DaisyUI-styled element has a non-default computed style); it fails because the runner and config do not exist
- [ ] 6.2 (GREEN) Add `@playwright/test`, a chromium-only `playwright.config.ts`, a Vite config that aliases `invoke` to a fixture-fed mock reading `tests/fixtures/`, and a `test:e2e` script; the layout test passes on chromium
- [ ] 6.3 (GREEN) Name e2e files `*.e2e.ts` and set `testMatch` so the native `bun test` runner does not collect them

## 7. CI and supersede enable-agent-testing

- [ ] 7.1 (GREEN) Add a chromium-only Playwright job to the test workflow (install chromium, run `bun test:e2e`); keep the staleness gate in the existing `cargo test` job and component tests in the `bun` job
- [ ] 7.2 Remove the `enable-agent-testing` change directory, since this change replaces it
- [ ] 7.3 Confirm `bun test`, `cargo test`, `bun lint`, and `bun test:e2e` (chromium) are green
