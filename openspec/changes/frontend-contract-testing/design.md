## Context

The app is a macOS-only Tauri v2 desktop application: a React/TypeScript frontend (Vite) over a Rust backend, with all external calls going through Tauri commands (`invoke`).
The generated bindings in `src/generated/tauri.ts` (tauri-specta) wrap `@tauri-apps/api/core`'s `invoke` and unwrap each result via `typedError`.
Existing tests are unit-only: Bun for TS (some service tests already mock `commands` via `mock.module`), Cargo for Rust including VCR cassette tests.

The goal is to merge agent-generated PRs without manual testing.
The unguarded gap is the frontend/backend contract: whether the frontend correctly consumes what the backend actually returns.
Hand-authored mocks do not guard this, because they can stay green when the backend shape changes.

Grounding from the backend:
- Daylite is structured for replay: `*_core` functions take an injected `DayliteApiClient`, `DayliteApiClient::with_env_cassette(base, "x.json")` replays a committed cassette with zero network, and the command wrappers return the `_core` result directly. Cassettes already exist.
- Holidays (`get_holidays_for_week`, hardcoded `date.nager.at` URL) and CalDAV/ZEP (`load_week_events`, `zep_discover_calendars`) build `reqwest` directly with no replay seam.
- Store and secret commands (`load_local_store`, `daylite_list_cached_contacts`, `zep_load_credentials`) are local.

## Goals / Non-Goals

**Goals:**
- Frontend component tests run under `bun test` with a DOM environment, fed by fixtures generated from the real backend.
- Fixtures are generated in Rust against the real types, committed, and guarded by a staleness gate so backend drift turns a test red.
- The frontend consumes fixtures through a typed `invoke` mock so the real `commands`/`typedError` code runs.
- The suite is fast and deterministic, with no browser, dev server, or network.

**Non-Goals:**
- Real-app E2E through WKWebView and the IPC bridge (flakiness and CI cost).
- Extending the replay seam to CalDAV/holidays (deferred; seeded fixtures for now).
- Real layout/CSS fidelity (manual spot checks on macOS).

## Decisions

### Component tests in `bun test`, not a browser
Given the priorities (fast, stable, no flakiness, browser/IPC out of scope, spot checks acceptable), component tests with a DOM emulator fit better than Playwright plus a dev server.
They run inside the existing `bun` job and the `Stop` hook, need no browser download, are millisecond-fast and deterministic, and run in the Claude cloud sandbox so changes can be self-verified in-session.
The accepted tradeoff is no real layout/CSS, which spot checks cover.
happy-dom is registered through a `bunfig.toml` test `preload`, and rendering/interaction use React Testing Library.

### Fixtures are generated in Rust, not hand-authored
The fixtures are serialized in Rust with serde, the same serialization tauri-specta derives the TS bindings from, so a fixture's JSON matches the shape the frontend receives.
Daylite fixtures are captured by calling each `_core` under `VCR_MODE=replay` against the committed cassettes, yielding recorded-real payloads.
Store, secret, holiday, and CalDAV fixtures are built from representative values of the real Rust structs (type-true but seeded), because those clients have no replay seam yet.
Either way the value is produced from the real types, so a type change forces the capture code to change and the committed fixture to be regenerated.

### Staleness gate lives inside `cargo test`
A normal (non-ignored) test regenerates the fixtures in memory under replay mode and compares them to the committed files, failing on any difference.
Regeneration is the same code path guarded by an `UPDATE_FIXTURES=1` env var that writes the files instead of comparing.
This puts the gate in the existing `cargo test` (which the rust CI job and the `Stop` hook already run, with cassettes unlocked via git-crypt), so no separate CI step is needed.
Serialization is deterministic (pretty JSON, stable field order; maps use ordered keys) so the comparison is stable.

### Frontend mock intercepts `invoke`, fed by fixtures
The mock replaces `@tauri-apps/api/core`'s `invoke` (via bun `mock.module`), so the real generated `commands` and `typedError` wrappers run against the fixture data, exercising the binding layer rather than bypassing it.
A typed registry maps each command's snake_case name to its fixture and is typed against the generated bindings (success-payload type per command), so a fixture that no longer matches a command's shape fails type checking.
This carries over the type-safety idea from `enable-agent-testing` but drops the `page.addInitScript` serialization boundary, since everything runs in-process.

### Fixtures shared between Rust and frontend
Captured JSON lives in `tests/fixtures/<command>.json`, written by the Rust capture and imported by the frontend mock.
One directory is the single source of truth, so the staleness gate and the frontend always read the same bytes.

### Scope of v1 commands
Only the commands the planning view exercises on load are captured first: `daylite_list_contacts` (recorded), `daylite_list_cached_contacts`, `load_local_store`, `load_week_events`, and `get_holidays_for_week` (seeded).
More commands and recorded-real upgrades for CalDAV/holidays are follow-on work.

## Risks / Trade-offs

- **Seeded fixtures are less real than recorded ones**: holidays/CalDAV/store fixtures are type-true but hand-seeded values.
  - **Mitigation**: the staleness gate still catches shape drift, Daylite (the most complex external) is recorded-real, and the seam can be extended later without changing the test shape.
- **serde vs specta serialization mismatch**: a fixture could in theory serialize differently from what the bindings expect.
  - **Mitigation**: the frontend mock is typed against the generated bindings, so a structural mismatch fails `tsc`; both derive from the same serde attributes.
- **happy-dom fidelity gap**: no real layout, some browser APIs are partial.
  - **Mitigation**: tests assert content and behavior, not pixels; macOS spot checks cover the visual layer.
- **Non-deterministic serialization would make the gate flaky**: unordered maps or timestamps would churn the diff.
  - **Mitigation**: stable ordering and no wall-clock values in fixtures (seed fixed dates).
