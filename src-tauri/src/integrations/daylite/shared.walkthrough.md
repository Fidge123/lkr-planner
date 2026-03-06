# Walkthrough: `src-tauri/src/integrations/daylite/shared.rs`

## Purpose

This file is the shared utility layer for the Daylite integration. It defines common DTOs, token helpers, JSON parsing helpers, and the standard Daylite error model.

## Block by block

### Imports (`lines 1-6`)

- The file depends on the local store module, Serde traits, `serde_json::Value`, `specta::Type`, and `SystemTime`.

### Shared DTOs (`lines 8-65`)

- `DayliteTokenState` stores the current access token, refresh token, and optional expiry timestamp.
- `DayliteTokenSyncStatus` is the small success response used by the refresh-token connect command.
- `DayliteSearchResult<T>` is a reusable generic wrapper for paginated search responses.
- `DayliteApiError` and `DayliteApiErrorCode` standardize error handling across all Daylite modules.
- `DayliteRefreshTokenRequest` and `DayliteSearchInput` are frontend command payloads.

Rust syntax to notice:
- Generic structs like `DayliteSearchResult<T>` let one response shape wrap many payload types.
- `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` ensures enum variants serialize as API-friendly constants.

Best practice:
- Centralize shared request and error types so modules cannot drift apart in behavior.

### Small utility helpers (`lines 67-114`)

- `build_limit_query` only emits a `limit` query parameter when one is present.
- `normalize_base_url` trims whitespace and trailing slashes and rejects an empty result.
- `load_daylite_tokens` and `store_daylite_tokens` translate between the store schema and the runtime token struct.
- `load_store_or_error` and `save_store_or_error` adapt local-store failures into the Daylite error type.

Rust syntax to notice:
- `if let Some(limit) = limit` is a direct way to handle optional inputs.
- Borrowing `&LocalStore` for reads and `&mut LocalStore` for writes makes ownership intent explicit.

### HTTP and JSON error normalization (`lines 116-186`)

- `normalize_http_error` maps status codes into domain-specific Daylite error codes and German user messages.
- `parse_success_json_body` enforces a 2xx status before delegating to the raw JSON parser.
- `parse_json_body` first parses into `serde_json::Value` and then deserializes into `T`, producing detailed technical messages for both failure stages.

Rust syntax to notice:
- `DeserializeOwned` is important for generic deserialization helpers that must return owned data.
- The code uses `format!` plus `truncate_for_log` to keep diagnostic messages useful without dumping arbitrarily large bodies.

Best practice:
- Keep user messages stable and friendly while still preserving detailed technical context for logs.

### Token and time helpers (`lines 188-244`)

- `missing_token_error` creates a consistent missing-credential error.
- `should_refresh_access_token` returns `true` when the access token is blank, missing an expiry, or within ten seconds of expiry.
- `current_epoch_ms` converts `SystemTime` into a `u64` millisecond timestamp with error handling.
- `truncate_for_log` limits logged bodies to 400 characters.
- `map_store_error` is the final adapter between local-store errors and Daylite errors.

Rust syntax to notice:
- `saturating_add(10_000)` is used to avoid overflow when computing the refresh buffer.
- `u64::try_from(...)` makes the integer conversion explicit and checked.

## Best practices this file demonstrates

- Hide repetitive parsing and error-mapping logic behind small shared helpers.
- Keep token state handling separate from endpoint-specific logic.
- Make helper functions return domain errors directly so callers stay focused on business flow.
