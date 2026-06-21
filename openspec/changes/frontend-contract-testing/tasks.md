## 1. Frontend DOM test environment

- [ ] 1.1 (RED) Add a component test `src/test/dom-env.spec.tsx` that renders a trivial element with React Testing Library and asserts it is in the document; it fails because no DOM environment or testing-library is configured
- [ ] 1.2 (GREEN) Add dev dependencies `@happy-dom/global-registrator`, `@testing-library/react`, `@testing-library/dom`
- [ ] 1.3 (GREEN) Create `bunfig.toml` with a `[test] preload` that registers happy-dom globally, and a setup module that installs RTL's auto-cleanup, so `bun test` runs with a DOM and the test passes

## 2. Fixture capture and staleness gate (Rust)

- [ ] 2.1 (RED) Add a Cargo test `command_fixtures_up_to_date` that, in `VCR_MODE=replay`, regenerates the planning-view command fixtures in memory and compares them to `tests/fixtures/*.json`; it fails because the generator and fixtures do not exist yet
- [ ] 2.2 (GREEN) Implement the fixture generator in Rust: capture `daylite_list_contacts` by calling `list_contacts_core` with `DayliteApiClient::with_env_cassette` against the committed cassette (recorded-real), and build type-true seeded values from the real structs for `load_local_store`, `daylite_list_cached_contacts`, `load_week_events`, and `get_holidays_for_week`
- [ ] 2.3 (GREEN) Serialize each command's success payload with serde to deterministic pretty JSON (stable field order, ordered map keys, fixed seed dates) so comparisons are stable
- [ ] 2.4 (GREEN) Make the test write the files instead of comparing when `UPDATE_FIXTURES=1`, generate the committed `tests/fixtures/*.json`, and confirm the comparison test passes; add a `fixtures:generate` script that runs it with `UPDATE_FIXTURES=1`
- [ ] 2.5 Confirm the gate fails on drift: changing a captured value without regenerating makes `cargo test` red

## 3. Typed fixture-fed invoke mock

- [ ] 3.1 (RED) Add a unit test for the mock asserting that `invoke("load_local_store")` resolves to the committed fixture and that an unregistered command throws a descriptive error; it fails because the mock does not exist yet
- [ ] 3.2 (GREEN) Create `src/test/tauri-fixture-mock.ts` that loads `tests/fixtures/*.json`, exposes an `invoke(command, args?)` returning the matching fixture (throwing for unmapped commands), and a helper to install it via `mock.module("@tauri-apps/api/core", ...)` so the real `commands`/`typedError` run against fixtures
- [ ] 3.3 (GREEN) Type the fixture registry against the generated bindings (success-payload type per command, snake_case keyed), so a fixture not assignable to its command's payload fails `tsc`

## 4. Planning-view component tests

- [ ] 4.1 (RED) Add `src/app.contract.spec.tsx` that installs the fixture mock, renders `<App />`, and asserts the planning view renders the fixture data (heading visible, fixture-derived content present) with no errors; it fails before the mock and fixtures are wired
- [ ] 4.2 (GREEN) Wire the fixtures so the planning view renders; assert at least one data-driven assertion beyond presence (for example a contact or holiday from the fixture appears)
- [ ] 4.3 (GREEN) Add an error-path test: a command fixture representing an error result renders the German error UI

## 5. Supersede enable-agent-testing

- [ ] 5.1 Remove the `enable-agent-testing` change directory, since this change replaces it
- [ ] 5.2 Confirm `bun test`, `cargo test`, and `bun lint` are green, and that the new tests run in the existing CI jobs without a browser
