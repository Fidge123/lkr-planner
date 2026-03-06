# Walkthrough: `src-tauri/src/integrations/daylite/recording_harness.rs`

## Purpose

This file contains a test-only harness for recording fresh Daylite HTTP cassettes from the live API. It is intentionally excluded from production builds and exists to keep replay fixtures current.

## Block by block

### Imports and environment variable names (`lines 1-18`)

- The harness reuses the real Daylite auth, client, contact, and project logic so recorded cassettes reflect true production behavior.
- The string constants define the environment variables required to run the ignored recording test.

Best practice:
- Keep environment variable names centralized so setup instructions and code cannot drift.

### Recording scope enum (`lines 20-40`)

- `DayliteVcrScope` supports `ReadOnly` and `All`.
- `from_env` reads `DAYLITE_VCR_SCOPE`, defaults to `"readonly"`, lowercases the input, and validates it.

Rust syntax to notice:
- `match ... { "readonly" => ..., value => Err(format!(...)) }` is a clean pattern for validating string enums.

### Recording configuration (`lines 42-75`)

- `DayliteVcrConfig` bundles all runtime inputs the recorder needs.
- `from_env` first enforces `VCR_MODE=record`.
- In read-only mode, mutation input is omitted.
- In `All` mode, contact update parameters become mandatory as well.

Best practice:
- Gather all environment parsing in one constructor so the actual recording flow can assume valid configuration.

### Ignored live-recording test (`lines 77-141`)

- `record_daylite_cassettes_from_live_api` is marked `#[ignore]` because it needs real credentials and writes files.
- The test refreshes tokens first, then records cassettes for project listing, project search, contact listing, and optionally contact iCal URL updates.
- It uses a synthetic `DayliteTokenState` with `u64::MAX` expiry so the later calls reuse a stable token instead of repeatedly refreshing.

Rust syntax to notice:
- `tauri::async_runtime::block_on` bridges async production code into a synchronous test function.
- `if let Some(update_contact_input) = ...` makes the mutation recording optional.

Best practice:
- Record cassettes by exercising real production code paths, not by scripting fake HTTP calls separately.

### Environment helpers (`lines 143-158`)

- `required_env` builds on `optional_env` and upgrades `None` into a readable error.
- `optional_env` reads the environment variable, trims whitespace, rejects blank values, and returns `Ok(None)` when the variable is missing.

Rust syntax to notice:
- `let Some(value) = ... else { return Ok(None); };` is the `let-else` syntax for early exits.

### Unit tests (`lines 160-253`)

- The tests verify default scope selection, `all` scope parsing, required mutation inputs, read-only behavior, and the `VCR_MODE=record` requirement.
- `env_lock` uses `OnceLock<Mutex<()>>` because tests mutate process-global environment variables.
- `clear_env` removes all known environment keys between test cases.

Rust syntax to notice:
- Environment mutation calls are wrapped in `unsafe` because Rust 2024 marks them unsafe due to potential data races with other threads.

## Best practices this file demonstrates

- Keep live fixture recording opt-in and clearly isolated.
- Serialize environment-variable parsing behind a dedicated config type.
- Guard global process state with a lock in tests.
