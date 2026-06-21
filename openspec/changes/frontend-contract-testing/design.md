## Context

The app is a macOS-only Tauri v2 desktop application: a React/TypeScript frontend (Vite) over a Rust backend, with all external calls going through Tauri commands (`invoke`).
The generated bindings in `src/generated/tauri.ts` (tauri-specta) wrap `@tauri-apps/api/core`'s `invoke` and unwrap each result via `typedError`.
Existing tests are unit-only: Bun for TS (some service tests already mock `commands` via `mock.module`), Cargo for Rust including VCR cassette tests.

The goal is to merge agent-generated PRs without manual testing.
The unguarded gap is the frontend/backend contract: whether the frontend correctly consumes what the backend actually returns, and whether the UI still renders.

Grounding from the backend:
- The record/replay machinery is generic at the file level (`http_record_replay.rs`: `RecordedRequest`/`RecordedResponse`), but the integration is `#[cfg(test)]` and Daylite-only: `DayliteApiClient` routes through a `DayliteHttpTransport` trait, and `ReqwestTransport` carries a `#[cfg(test)] record_replay: Option<RecordReplayConfig>` that short-circuits to replay/record.
- Holidays (`fetch_from_url`, hardcoded `date.nager.at`) and CalDAV/ZEP (`load_week_events`) build `reqwest` inline with no transport seam.
- The command wrappers return the `_core` result directly (store writes are side effects), so a command's frontend-visible payload equals its core output.

## Goals / Non-Goals

**Goals:**
- Holidays and CalDAV/ZEP can record and replay cassettes through a shared HTTP transport, so their fixtures are recorded-real.
- Frontend component tests run under `bun test` in a DOM environment, fed by fixtures generated from the real backend, guarded by a staleness gate.
- A minimal chromium Playwright test confirms the app renders with its CSS/layout.
- The suite is fast and deterministic, with no network and no IPC/WebDriver layer.

**Non-Goals:**
- Real-app E2E through WKWebView and the IPC bridge.
- Cross-engine layout verification in CI (chromium only; macOS spot checks for WKWebView).
- Visual regression / screenshot diffing.

## Decisions

### Generalize the record/replay transport, do not duplicate it per client
Rather than copy the Daylite `#[cfg(test)]` transport into holidays and CalDAV, extract a single reusable record/replay HTTP transport keyed on method/path/query/body, and have Daylite, holidays, and CalDAV/ZEP all route through it.
This keeps one cassette format and one replay code path, and turns "add a new recorded integration" into "route its requests through the shared transport".
The transport stays `#[cfg(test)]` so production builds are unaffected, matching the current Daylite design.

### Holidays and ZEP/CalDAV fixtures are recorded-real
With the shared transport in place, holidays and CalDAV/ZEP get recording harnesses (live, `#[ignore]`, like the Daylite harness) and committed cassettes, so their fixtures are captured under `VCR_MODE=replay` from real recorded responses.
Holidays uses the public Nager API, so it records without credentials.
ZEP/CalDAV requires live credentials and a reachable server, so recording its cassette is a one-time manual step; everything else (seam, replay, capture, tests) is independent of live access.
Only the pure-local commands (`load_local_store`, `daylite_list_cached_contacts`, `zep_load_credentials`) remain type-true seeded, because they read the local store / secret store rather than HTTP.

### Fixtures are generated in Rust with a staleness gate in cargo test
Fixtures are serialized in Rust with serde, the same serialization tauri-specta derives the bindings from, so the JSON matches the shape the frontend receives.
A normal `cargo test` regenerates the fixtures in memory under replay mode and compares them to the committed `tests/fixtures/*.json`, failing on any difference; the same path writes the files when `UPDATE_FIXTURES=1`.
This puts the gate in the existing `cargo test` (which CI and the `Stop` hook already run with cassettes unlocked), with no separate CI step.
Serialization is deterministic (pretty JSON, stable field order, ordered map keys, fixed seed dates) so comparisons are stable.

### Frontend mock intercepts invoke, fed by fixtures
The mock replaces `@tauri-apps/api/core`'s `invoke` (bun `mock.module`), so the real generated `commands` and `typedError` run against the fixture data, exercising the binding layer rather than bypassing it.
A typed registry maps each command's snake_case name to its fixture and is typed against the generated bindings (success-payload type per command), so a fixture not assignable to a command's shape fails type checking.
Component rendering and interaction use React Testing Library; happy-dom is registered through a `bunfig.toml` test `preload`.

### A minimal Playwright layout check on chromium
happy-dom has no real layout, so one Playwright test on chromium renders the app (Vite dev server with the same fixtures aliased in for `invoke`) and asserts the basics: key regions are visible, have non-zero size, and CSS is actually applied (for example a DaisyUI-styled element has a non-default computed style).
It is deliberately one small test, not a visual-regression suite, to keep flakiness and maintenance low while letting an agent see that the UI is not broken.
chromium is pre-staged in the Claude cloud sandbox, so the agent can run it; webkit is left to manual spot checks per the macOS-only engine note.

### Fixtures shared between Rust, component tests, and Playwright
Captured JSON lives in `tests/fixtures/<command>.json`, written by the Rust capture and read by both the component-test mock and the Playwright mock, so all three layers and the staleness gate read the same bytes.

### Scope of v1 commands
Captured first: `daylite_list_contacts` and `get_holidays_for_week` and `load_week_events` (recorded), plus `load_local_store` and `daylite_list_cached_contacts` (seeded).
More commands and recorded upgrades follow the same pattern.

## Risks / Trade-offs

- **ZEP/CalDAV recording needs live access**: the cassette cannot be recorded in CI or the sandbox.
  - **Mitigation**: it is a one-time manual `VCR_MODE=record` run; the rest of the work and all replay tests are independent of it, and the committed cassette is reused thereafter.
- **Generalizing the transport touches working integrations**: refactoring Daylite/holidays/CalDAV onto a shared transport risks regressions.
  - **Mitigation**: keep the existing Daylite cassettes and tests green throughout as a regression guard, and move one integration at a time.
- **happy-dom fidelity gap**: no real layout in component tests.
  - **Mitigation**: the chromium Playwright test covers basic layout/CSS; macOS spot checks cover engine differences.
- **Playwright reintroduces a small flakiness surface**: one browser test plus a dev server.
  - **Mitigation**: a single deterministic assertion set fed by fixtures, chromium only, with no network.
- **Non-deterministic serialization would make the gate flaky**.
  - **Mitigation**: stable ordering and fixed seed dates; no wall-clock values in fixtures.
