# Walkthrough: `integrations/daylite/mod.rs`

This file serves as the module declaration point for the `daylite` subsystem. Because it resides in `integrations/daylite/mod.rs`, it represents the `daylite` module defined back in `integrations/mod.rs`.

```rust
pub mod auth;
mod auth_flow;
mod client;
pub mod contacts;
pub mod projects;
pub mod shared;
```
Here, we expose distinct internal modules for the Daylite API integration. Notice that some are `pub mod` while others are just `mod`.

- `pub mod auth`, `contacts`, `projects`, `shared`: These are public. Anything calling the `daylite` module from the outside (like `lib.rs`) is allowed to see and interact with these components directly. `lib.rs` exports their commands directly to Tauri.
- `mod auth_flow` and `mod client`: These lack the `pub` keyword, making them private. They handle sensitive backend business logic (like HTTP clients and auth loop behaviors) that should only be visible internally mapped inside the `daylite` module.
