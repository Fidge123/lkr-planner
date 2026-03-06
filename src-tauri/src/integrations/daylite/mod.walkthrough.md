# Walkthrough: `src-tauri/src/integrations/daylite/mod.rs`

## Purpose

This file is the namespace root for the Daylite integration. It decides which submodules are public API, which are internal plumbing, and which helpers exist only for tests.

## Block by block

### Module declarations (`lines 1-8`)

- `pub mod auth;`, `pub mod contacts;`, `pub mod projects;`, and `pub mod shared;` are the public Daylite-facing modules.
- `mod auth_flow;` and `mod client;` are internal helpers used by the public modules.
- `#[cfg(test)] mod recording_harness;` keeps the live-cassette recorder out of production builds.

Rust syntax to notice:
- `pub mod` exports a module.
- Plain `mod` keeps the module private to its parent unless specific items are re-exported elsewhere.
- `#[cfg(test)]` is a compile-time gate, not a runtime `if`.

## Best practices this file demonstrates

- Keep the public API surface small.
- Separate command handlers from transport and auth plumbing.
- Keep test-only tooling out of release binaries.
