# Walkthrough: `src-tauri/src/integrations/health.rs`

## Purpose

This file exposes a simple Tauri command that reports whether the backend is alive, plus metadata the frontend can display or log.

## Block by block

### Imports (`lines 1-2`)

- `serde::{Deserialize, Serialize}` lets the structs move between Rust and JSON.
- `specta::Type` lets Specta generate matching TypeScript types.

### Response types (`lines 4-17`)

- `HealthStatus` is the command payload returned to the frontend.
- `HealthStatusEnum` models the status as an enum instead of a raw string.
- `#[serde(rename_all = "lowercase")]` serializes `Healthy` as `"healthy"` and `Unhealthy` as `"unhealthy"`.

Rust syntax to notice:
- `#[derive(...)]` asks Rust macros to generate common trait implementations.
- Enums are a better fit than free-form strings when the value set is fixed.

Best practice:
- Prefer structured types over ad-hoc JSON so the frontend gets compile-time help.

### Tauri command (`lines 19-31`)

- `#[tauri::command]` exposes the function to the frontend.
- `#[specta::specta]` adds metadata so the command can be included in generated TypeScript bindings.
- The function reads the current UTC time with `chrono::Utc::now()` and the package version with `env!("CARGO_PKG_VERSION")`.
- It always returns a healthy result right now.

Rust syntax to notice:
- `env!` is a compile-time macro, not a runtime environment lookup.
- `Result<HealthStatus, String>` leaves room for future failure cases even though the function currently succeeds.

### Test module (`lines 33-53`)

- `#[cfg(test)]` makes the module compile only during `cargo test`.
- The first test checks shape and basic content.
- The second test verifies that the returned version matches the package version embedded at compile time.

Rust syntax to notice:
- `matches!(...)` is a macro for readable enum matching in assertions.
- `unwrap()` is acceptable in tests when failure should immediately fail the test.

## Best practices this file demonstrates

- Keep health checks predictable and dependency-light.
- Return structured metadata that is useful for both UI and diagnostics.
- Test the public command behavior, not just internal helpers.
