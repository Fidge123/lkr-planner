# Walkthrough: `integrations/mod.rs`

This file serves as the entry point for the `integrations` module.

```rust
pub mod daylite;
pub mod health;
pub mod local_store;
```
In Rust, whenever a folder is declared as a module (e.g., `mod integrations;` in `lib.rs`), Rust looks for `integrations/mod.rs` to understand what that module physically contains.

The `pub mod` declarations instruct the compiler that three separate components map inside the `integrations/` folder namespaces. These represent adjacent files (e.g. `health.rs`, `local_store.rs`) or entire directories wrapping to `daylite/mod.rs`. 
The `pub` keyword makes these submodules public. Because Rust module scope defaults to `private` out of self-preservation, this instructs the application structure to map these folders outward so they correspond appropriately to external dependencies!
