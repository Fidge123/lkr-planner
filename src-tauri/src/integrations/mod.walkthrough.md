# Walkthrough: `src-tauri/src/integrations/mod.rs`

## Purpose

This module is the namespace root for backend integrations. It exposes the production-facing modules and hides test-only helpers behind conditional compilation.

## Block by block

### Module-level documentation (`lines 1-11`)

- The `///` comments are Rust doc comments. They describe the architectural role of the integration layer.
- The key idea is that Rust owns secrets, network calls, and external API coordination, while the frontend talks to Rust through Tauri commands.

Rust syntax to notice:
- `///` attaches documentation to the next item.
- These comments can be rendered by `cargo doc`.

### Public module exports (`lines 12-16`)

- `pub mod daylite;`, `pub mod health;`, and `pub mod local_store;` make those modules available outside this module tree.
- `#[cfg(test)] pub(crate) mod http_record_replay;` compiles the cassette helper only for tests and keeps it visible only inside the current crate.

Rust syntax to notice:
- `pub` means visible to other modules and crates that can reach this module.
- `pub(crate)` means visible anywhere inside the current crate, but not outside it.
- `#[cfg(test)]` strips code out of non-test builds entirely.

## Best practices this file demonstrates

- Use the module root to enforce architecture boundaries.
- Keep test infrastructure out of production binaries with `#[cfg(test)]`.
