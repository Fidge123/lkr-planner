# Walkthrough: `main.rs`

This is the entry point for the rust side of the Tauri application.

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```
This is a rust attribute. `#![...]` applies the attribute to the whole file/crate (because of the `!`). `cfg_attr` conditionally compiles an attribute. Here, if we are *not* in debug mode (`not(debug_assertions)`), it sets the Windows subsystem to "windows", which hides the terminal window that would otherwise pop up when running a graphical application on Windows.

```rust
fn main() {
    lkr_planner_lib::run()
}
```
This is the main function. It simply delegates execution to the `run` function defined in the library crate (`lib.rs`). This is a common pattern in Rust (and Tauri): keeping `main.rs` extremely thin and putting the actual initialization and application logic in `lib.rs`, making the code more easily testable or reusable like a typical library.
