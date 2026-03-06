# Walkthrough: `src-tauri/src/main.rs`

## Purpose

This file is the thin executable entry point. It applies one Windows-specific build attribute and then hands control to the library crate.

## Block by block

### Windows subsystem attribute (`lines 1-2`)

- The comment warns that the next line is intentional.
- `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` is a crate-level attribute.
- In release builds on Windows, it prevents an extra console window from appearing next to the desktop app.

Rust syntax to notice:
- `#![...]` applies an inner attribute to the whole crate, not just the next item.
- `not(debug_assertions)` flips the condition, so the attribute is skipped in debug builds.

### Delegating `main` function (`lines 4-6`)

- `fn main()` is the executable entry point the operating system launches.
- `lkr_planner_lib::run()` delegates to the library crate, where the real Tauri setup lives.

Rust syntax to notice:
- `crate_name::function()` calls into another crate target from the same package.
- The function body has only one expression, so the file stays intentionally minimal.

## Best practices this file demonstrates

- Keep `main.rs` tiny so platform quirks stay isolated.
- Put application logic in `lib.rs`, which is easier to test and reuse.
