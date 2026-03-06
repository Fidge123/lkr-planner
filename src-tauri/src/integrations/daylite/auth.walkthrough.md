# Walkthrough: `src-tauri/src/integrations/daylite/auth.rs`

## Purpose

This file exposes the Tauri command that accepts a Daylite base URL plus refresh token, refreshes credentials immediately, and stores the resulting token state in the local store.

## Block by block

### Imports (`lines 1-6`)

- `refresh_tokens` performs the OAuth-like refresh flow.
- `DayliteApiClient` sends HTTP requests.
- The shared helpers load and save the local store, normalize the base URL, and copy token fields into the persisted store.

### Tauri command (`lines 8-27`)

- `daylite_connect_refresh_token` is the frontend entry point.
- The command first normalizes the base URL, then creates a client for that URL.
- It calls `refresh_tokens` with the provided refresh token.
- On success, it loads the local store, writes the normalized base URL and refreshed token state back into it, saves the store, and returns booleans describing whether access and refresh tokens are now present.

Rust syntax to notice:
- `pub async fn ... -> Result<..., DayliteApiError>` is the standard shape for an async command that can fail.
- `app.clone()` is needed because the same `AppHandle` is used for both reading and writing.
- `?` keeps the function linear by returning early on any failure.

Best practice:
- Validate and normalize inputs before persisting them.
- Refreshing immediately is safer than storing an unverified refresh token.

## Best practices this file demonstrates

- Keep command handlers thin and delegate domain logic to helpers.
- Persist only verified token state.
