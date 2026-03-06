# Walkthrough: `src-tauri/src/integrations/local_store.rs`

## Purpose

This file defines the JSON-backed local configuration store for the desktop app. It owns the on-disk schema, Tauri commands for loading and saving it, and the error model used when persistence fails.

## Block by block

### Imports and file name constant (`lines 1-7`)

- `serde` and `specta` derive support for JSON and TypeScript generation.
- `std::fs` and `std::path::Path` handle file I/O.
- `tauri::Manager` is needed so `AppHandle` can resolve the app config directory.
- `STORE_FILE_NAME` keeps the filename in one place.

### Root store type and default (`lines 9-33`)

- `LocalStore` is the top-level persisted document.
- The manual `Default` implementation composes defaults from nested types and initializes list fields to empty vectors.

Rust syntax to notice:
- `#[serde(rename_all = "camelCase")]` keeps the JSON schema aligned with frontend naming conventions.
- A manual `impl Default` is useful when the default is more meaningful than "empty for every field."

Best practice:
- Define one root persisted type so versioning and migrations stay manageable.

### Nested configuration types (`lines 35-144`)

- `ApiEndpoints` stores service base URLs.
- `TokenReferences` stores token material and token-related metadata.
- `EmployeeSetting`, `StandardFilter`, `ContactFilter`, and `RoutingSettings` represent separate feature areas.
- `DayliteCache` and its nested cache entry types store the latest synced Daylite data for offline reuse.

Rust syntax to notice:
- `#[specta(type = Option<f64>)]` overrides the generated TypeScript type for `Option<u64>` so JavaScript receives a safe numeric shape.
- `#[derive(Default)]` is used when the type's empty state is already correct.

Best practice:
- Split a large persisted schema into focused structs so future changes stay localized.

### Store error model (`lines 146-161`)

- `StoreError` separates a machine-readable `code` from a German `user_message` and an English-style technical diagnostic.
- `StoreErrorCode` classifies failure modes such as read errors, write errors, corrupt JSON, and missing required fields.

Best practice:
- Distinguish user-facing text from developer-facing diagnostics.

### Public Tauri commands (`lines 163-193`)

- `load_local_store` resolves the app config directory, appends `local-store.json`, and delegates to `load_store_from_path`.
- `save_local_store` does the same for writes and delegates to `save_store_to_path`.
- Both commands convert path-resolution failures into structured `StoreError` values.

Rust syntax to notice:
- `app.path().app_config_dir()` returns a `Result`, so `.map(...)` and `.map_err(...)` are used to transform success and failure branches.
- `?` exits early with the error if path resolution fails.

### Internal load helper (`lines 195-227`)

- If the file does not exist, the function returns `LocalStore::default()` instead of treating that as an error.
- `fs::read_to_string` loads the JSON document.
- `serde_json::from_str::<LocalStore>` deserializes the content.
- The code treats `"missing field"` errors specially and maps them to `StoreErrorCode::MissingFields`; other parse problems become `CorruptFile`.

Rust syntax to notice:
- `map_err(|error| ...)` is a common Rust pattern for translating low-level errors into domain errors.
- `return Ok(...)` is used for the early "file missing" success path.

Best practice:
- Missing config files are usually first-run state, not failures.
- One caution: the missing-field classification depends on Serde's current error wording, so that branch is pragmatic but somewhat brittle.

### Internal save helper (`lines 229-257`)

- The function creates parent directories when needed.
- `serde_json::to_string_pretty` keeps the stored file human-readable.
- `fs::write` persists the serialized store.

Rust syntax to notice:
- `if let Some(parent) = path.parent()` safely handles paths that may not have a parent.
- The final `Ok(())` is the conventional way to signal success for a side-effect-only function.

Best practice:
- Persist pretty-printed JSON when operators may need to inspect the file manually.

### Tests (`lines 259-391`)

- The tests cover first-run defaults, save/load round-tripping, corrupt JSON handling, and missing-field handling.
- `unique_test_path` creates isolated temp paths using nanosecond timestamps.
- `write_test_file` is a small helper that creates parent directories before writing fixtures.

Rust syntax to notice:
- `std::env::temp_dir()` gives a safe temp base path.
- `assert_eq!` works because the data types derive `PartialEq` and `Eq`.

## Best practices this file demonstrates

- Keep persisted schema definitions explicit and typed.
- Prefer structured error values over string-only failures.
- Treat missing config as a normal bootstrap case.
- Test persistence code against real files, not only mocked helpers.
