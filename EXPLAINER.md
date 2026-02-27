# Rust Explainer for This Branch

This guide explains the new Rust code added on your current branch and teaches the Rust syntax and idioms used there, with comparisons to TypeScript where useful.

## 1) What changed in Rust on this branch

### New files

- `src-tauri/src/integrations/daylite/mod.rs`
  - Declares the Daylite submodules.
- `src-tauri/src/integrations/daylite/auth.rs`
  - Tauri command for connecting a Daylite refresh token and storing rotated tokens.
- `src-tauri/src/integrations/daylite/projects.rs`
  - Tauri commands to list/search Daylite projects.
- `src-tauri/src/integrations/daylite/contacts.rs`
  - Tauri commands to list/search contacts and update iCal URLs on a contact.
- `src-tauri/src/integrations/daylite/shared.rs`
  - Shared DTOs, error model, token helpers, and store bridge helpers.
- `src-tauri/src/integrations/daylite/client.rs`
  - Core Daylite API client, transport abstraction, token refresh logic, response parsing, and extensive tests.

### Modified files

- `src-tauri/src/lib.rs`
  - Registers new Daylite Tauri commands via `tauri_specta`.
- `src-tauri/src/integrations/mod.rs`
  - Exposes the new `daylite` module.
- `src-tauri/src/integrations/local_store.rs`
  - Extends persisted schema with Daylite access/refresh/expiry token fields and richer cached contact URL fields.

## 2) Architecture and request flow

High-level flow for a command like `daylite_list_projects`:

1. Frontend calls a Tauri command.
2. Command loads persisted `LocalStore`.
3. Command builds `DayliteApiClient` with configured base URL.
4. Command loads token state from store.
5. Client checks if access token is missing/near expiry; refreshes if needed.
6. Client sends Daylite HTTP request through transport.
7. Client normalizes errors and deserializes JSON into typed structs.
8. Command writes rotated token state back to store.
9. Typed data is returned to the frontend.

This makes Rust the single place for auth/token/network concerns, while TypeScript receives already-normalized domain data/errors.

## 3) Rust feature vs library code (important distinction)

### Core Rust language / std features in this code

- `mod`, `pub`, `pub(super)` module visibility
- `struct`, `enum`, `impl`, `trait`
- `Option<T>`, `Result<T, E>`
- Pattern matching: `match`, `if let`, `let Some(x) = ... else { ... }`
- Generics and trait bounds: `fn execute_json_request<T: DeserializeOwned>(...)`
- Ownership/borrowing: `String` vs `&str`, cloning where needed
- `async fn` and `.await`
- Smart pointer from std: `Arc<T>`
- `derive` for standard traits: `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`
- `?` operator for early error return

### External library features in this code

- `#[tauri::command]`, `tauri::AppHandle` (Tauri)
- `#[specta::specta]`, `specta::Type`, `tauri_specta` export (Specta)
- `serde::{Serialize, Deserialize}` and `#[serde(...)]` field mapping (Serde)
- `serde_json::{Value, json!}` for dynamic JSON handling (Serde JSON)
- `tauri_plugin_http::reqwest` HTTP client (Tauri HTTP plugin / Reqwest)

Rule of thumb:
- If it looks like language syntax (`match`, `Result`, `impl`, lifetimes), it is Rust.
- If it looks like attribute macros with crate names (`#[tauri::command]`, `#[serde(...)]`), it is library behavior layered on top of Rust.

## 4) File-by-file walkthrough

### `src-tauri/src/integrations/daylite/shared.rs`

Key responsibilities:

- Defines shared DTOs:
  - `DayliteTokenState`
  - `DayliteSearchResult<T>`
  - `DayliteApiResponse<T>`
  - `DayliteApiError` and `DayliteApiErrorCode`
- Normalizes config and HTTP failures:
  - `normalize_base_url`
  - `normalize_http_error`
- Bridges token state <-> local store:
  - `load_daylite_tokens`
  - `store_daylite_tokens`
- Utility logic:
  - `build_limit_query`
  - `should_refresh_access_token`
  - `current_epoch_ms`
  - `truncate_for_log`

Rust idioms shown:

- `Option`-aware control flow.
- Strongly typed error enum codes instead of magic strings.
- `map_err` to convert low-level errors into domain errors.

### `src-tauri/src/integrations/daylite/client.rs`

This is the core.

- `DayliteApiClient` methods map to Daylite operations.
- `execute_json_request<T>` centralizes:
  - token checks
  - preemptive token refresh
  - request execution
  - status handling
  - JSON parsing + typed deserialization
- `refresh_tokens` handles refresh rotation and expiry calculation.
- `DayliteHttpTransport` trait abstracts transport.
- `ReqwestTransport` is production implementation.
- `MockTransport` in tests validates behavior without real network.

Rust idioms shown:

- Trait-based dependency inversion for testability.
- Generic deserialization (`T: DeserializeOwned`).
- Borrowing vs owning inputs (`&str` for inputs, `String` when storing).
- `saturating_add` / `saturating_mul` defensive arithmetic.

### `src-tauri/src/integrations/daylite/projects.rs`

- Defines `DayliteProjectSummary` with Serde aliases/defaults.
- Commands:
  - `daylite_list_projects`
  - `daylite_search_projects`
- Pattern:
  - load store -> call client -> persist token state -> return data.

### `src-tauri/src/integrations/daylite/contacts.rs`

- Defines `DayliteContactSummary`, `DayliteContactUrl`, input DTO for updates.
- Commands:
  - `daylite_list_contacts`
  - `daylite_search_contacts`
  - `daylite_update_contact_ical_urls`
- Same store/client/token lifecycle pattern as projects.

### `src-tauri/src/integrations/daylite/auth.rs`

- Command `daylite_connect_refresh_token`:
  - validates base URL
  - exchanges refresh token for access+refresh+expiry
  - persists base URL and tokens
  - returns sync status booleans

### `src-tauri/src/integrations/local_store.rs` (changes)

New schema fields:

- `tokenReferences.dayliteAccessToken`
- `tokenReferences.dayliteRefreshToken`
- `tokenReferences.dayliteAccessTokenExpiresAtMs`
- richer cached contact URL fields
- rename compatibility from `projectProposalFilters` to `standardFilter` via serde alias

Important compatibility choices:

- `#[serde(default)]` prevents older JSON from failing due to new fields.
- `#[serde(alias = "...")]` supports renamed field migration.

## 5) Simplified vs production examples

### A) Generic typed request pipeline

Simplified:

```rust
async fn get_json<T: serde::de::DeserializeOwned>(body: &str) -> Result<T, String> {
    serde_json::from_str(body).map_err(|e| e.to_string())
}
```

Production adds complexity because it must also:

- validate auth/token presence
- refresh token preemptively
- call HTTP transport
- normalize status-specific user+technical errors
- return updated token state with the data

### B) Why `BoxFuture` + trait object is used

Simplified (TypeScript style idea):

```ts
interface Transport {
  send(req: Request): Promise<Response>;
}
```

Rust equivalent in this code:

```rust
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait DayliteHttpTransport: Send + Sync {
    fn send<'a>(&'a self, request: DayliteHttpRequest)
        -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>>;
}
```

Why complex:

- Rust trait objects need explicit erased future type (`Pin<Box<dyn Future...>>`) for async methods in object-safe trait usage.
- Benefit: `DayliteApiClient` can use real HTTP in production and mock transport in tests without changing business logic.

### C) Token refresh strategy

Simplified:

```rust
if token_expired {
    token = refresh(token.refresh_token).await?;
}
```

Production adds:

- refresh also when token is missing or <= 10s from expiry
- reject empty refresh/access token responses
- compute `expires_at_ms` from `expires_in`
- use saturating arithmetic to avoid overflow edge cases
- persist rotated refresh token after each call path

### D) URL merge logic for contacts

Simplified:

```rust
fn set_ical_urls(urls: Vec<Url>, primary: &str, absence: &str) -> Vec<Url> {
    vec![mk("Einsatz iCal", primary), mk("Abwesenheit iCal", absence)]
}
```

Production needs to:

- preserve unrelated existing URLs
- remove only previous iCal-like labels (including legacy labels like "Fehlzeiten")
- normalize labels (trim/lowercase) before matching
- avoid writing empty URL strings

### E) Serde mapping and schema migration

Simplified:

```rust
#[derive(Serialize, Deserialize)]
struct Contact {
    name: String
}
```

Production mapping examples:

- `#[serde(rename = "self")]` maps JSON field `self` to Rust field `reference`.
- `#[serde(alias = "first_name")]` accepts alternative input key names.
- `#[serde(default)]` fills missing optional/new fields.
- `#[serde(skip_serializing_if = "Option::is_none")]` avoids writing null-ish fields unnecessarily.

## 6) Rust syntax quick map for a TypeScript developer

- `String` ~= owned mutable heap string, `&str` ~= borrowed immutable string slice
- `Option<T>` ~= `T | undefined` but explicit and exhaustively handled
- `Result<T, E>` ~= `Promise<T>` that cannot throw implicitly; error type is explicit
- `?` ~= propagate error early (like `throw` in a controlled typed channel)
- `impl Type { ... }` ~= class methods block (without inheritance)
- `trait` ~= interface (can include behavior contracts)
- `enum` ~= tagged union / discriminated union

Example:

```rust
fn parse_limit(input: Option<u16>) -> Result<u16, String> {
    match input {
        Some(v) if v > 0 => Ok(v),
        Some(_) => Err("must be > 0".to_string()),
        None => Ok(25),
    }
}
```

## 7) Idioms and best practices you can reuse

- Centralize error normalization close to integration boundary.
- Keep Tauri command handlers thin; put API logic in a client module.
- Use typed DTOs plus serde attributes at API boundaries.
- Use traits + mocks for deterministic tests.
- Persist rotated token state immediately after calls.
- Use `#[serde(default)]` and aliases for backwards-compatible local-store evolution.

## 8) Reading order to learn quickly

1. `src-tauri/src/lib.rs` to see command surface.
2. `src-tauri/src/integrations/daylite/projects.rs` and `contacts.rs` for command pattern.
3. `src-tauri/src/integrations/daylite/shared.rs` for shared models/errors/helpers.
4. `src-tauri/src/integrations/daylite/client.rs` for core networking/token logic.
5. `src-tauri/src/integrations/local_store.rs` for persistence schema and compatibility.
6. `src-tauri/src/integrations/daylite/client.rs` tests to understand behavior guarantees.

## 9) Why this design is "more complex than minimal", and why that is reasonable

The complexity mainly comes from non-trivial integration requirements, not Rust for Rust's sake:

- Daylite token rotation and expiry handling
- Persisted token state across app restarts
- Unified error contract for frontend UX
- Schema compatibility in local persisted store
- Testable networking via transport abstraction

A minimal version would be much shorter but fragile. The current design trades some verbosity for reliability and deterministic behavior under real API conditions.
