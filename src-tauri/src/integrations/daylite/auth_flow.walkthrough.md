# Walkthrough: `src-tauri/src/integrations/daylite/auth_flow.rs`

## Purpose

This file owns the Daylite authentication workflow after a refresh token already exists. It decides when to refresh, performs the refresh request, and provides a generic helper for sending authenticated JSON requests.

## Block by block

### Imports (`lines 1-10`)

- The file depends on the Daylite client, HTTP method enum, and shared error/token helpers.
- `DeserializeOwned` and `serde_json::Value` make the request helper generic over any JSON response body.

Rust syntax to notice:
- `T: DeserializeOwned` means "any type that can be fully deserialized without borrowing from the input string."

### `ensure_access_token` (`lines 12-29`)

- This function accepts a possibly stale `DayliteTokenState`.
- If both access and refresh tokens are blank, it returns a missing-token error immediately.
- Otherwise it reads the current time and asks `should_refresh_access_token` whether a refresh is needed.
- If so, it replaces the token state by awaiting `refresh_tokens`.

Rust syntax to notice:
- `mut token_state` means the local binding can be reassigned.
- Returning the token state, even when unchanged, makes the caller's next step explicit.

Best practice:
- Centralize refresh policy in one helper so other request code stays simple.

### `refresh_tokens` (`lines 31-110`)

- Blank refresh tokens are rejected before any network call.
- The function sends `GET /personal_token/refresh_token` with the refresh token as a query parameter.
- Non-2xx responses are normalized into `DayliteApiError`, then reclassified specifically as `TokenRefreshFailed`.
- The body is parsed into `DayliteRefreshTokenResponse`.
- The code trims and validates `access_token`, `refresh_token`, and `expires_in`.
- Finally it computes `access_token_expires_at_ms` with `saturating_add` and `saturating_mul` to avoid integer overflow.

Rust syntax to notice:
- `if !(200..300).contains(&response.status)` is a readable range check.
- `format!(...)` builds detailed technical messages without mixing them into the user-facing text.
- `Some(expires_at_ms)` wraps the computed timestamp in an `Option`.

Best practice:
- Validate server responses defensively even after successful HTTP status codes.

### `send_authenticated_json` (`lines 112-133`)

- This generic helper is the normal path for authenticated Daylite requests.
- It first ensures an access token exists and is fresh.
- It then forwards the request through the client, attaching the access token.
- `parse_success_json_body::<T>` validates the HTTP status and deserializes the response into the caller's target type.
- The function returns both the parsed payload and the possibly refreshed token state.

Rust syntax to notice:
- Returning `(T, DayliteTokenState)` is a tuple return type.
- `::<T>` makes the generic type parameter explicit for the parser call.

Best practice:
- Return updated auth state together with the payload so callers cannot forget to persist refreshed tokens.

### Refresh DTO and parser (`lines 135-153`)

- `DayliteRefreshTokenResponse` is private because only this file needs it.
- `parse_refresh_response_body` reuses the generic JSON parser but rewrites any parse failure into the refresh-specific error code and message.

Rust syntax to notice:
- A private `struct` is often the cleanest way to model a one-endpoint response.
- `map_err(...)` is used here to remap a shared error into a more precise domain error.

### Tests (`lines 155-520`)

- The tests cover missing tokens, token reuse, refresh validation, request behavior, malformed responses, VCR replay, and the mock transport implementation.
- `MockTransport` stores queued responses in a `VecDeque` and records all outgoing requests for assertions.
- `tauri::async_runtime::block_on` lets plain unit tests execute async code.

Rust syntax to notice:
- `Arc<Mutex<...>>` is a standard pattern for shared mutable test state.
- `Box::pin(async move { ... })` is how the mock implements the transport's boxed-future API.

Best practice:
- Test both success paths and defensive validation branches for auth code.

## Best practices this file demonstrates

- Keep refresh logic separate from endpoint-specific business logic.
- Preserve user-friendly and technical error messages separately.
- Use small generic helpers to eliminate repeated auth boilerplate.
