# ADR 0006: Daylite Access/Refresh Token Rotation and Persistence

- Status: Accepted
- Date: 2026-02-14
- Amended: 2026-08-14 (token storage moved to the keychain, rotation narrowed to a session lock, `401` retry added)
- Amended: 2026-10-16 (rotated tokens survive a refused keychain write, exit waits for an in-flight rotation)

## Context

BL-006 requires a minimal Daylite API client for project/contact read and search operations with normalized errors.
Daylite issues short-lived access tokens and single-use refresh tokens.
Without persistence of rotated tokens and access token expiry, authenticated requests will fail after restart or token rotation.

## Decision

- Implement the Daylite API client in Rust (`src-tauri/src/integrations/daylite/`) behind Tauri commands.
- Use the Daylite refresh endpoint (`/personal_token/refresh_token`) with query parameter `refresh_token`.
- Normalize all Daylite failures into one typed error payload containing:
  - machine-readable error code
  - optional HTTP status
  - German user-facing message
  - technical debug message
- Persist the latest rotated Daylite tokens, including the access token expiry, in the OS keychain (see ADR on secure token storage).
- On Daylite requests:
  - refresh access token only when it is expired or within 10 seconds of expiry
  - compute access-token expiry using `expires_in`
  - always replace and persist the refresh token returned by the refresh call
  - send the current access token for API requests

## Amendment 2026-08-14: session-scoped rotation

The original implementation held one process-wide lock across the whole load, request and store cycle of every Daylite call.
That was wider than the invariant requires and made each call pay a keychain read and a keychain write, while serializing unrelated reads behind each other.

- Hold the token state in memory for the process lifetime, seeded from the keychain on first use.
- Serialize rotation only.
  Sending a request with an already valid access token is safe in parallel, because an access token is a bearer credential; only rotation cannot race, since Daylite invalidates a refresh token as it issues the next one.
- Re-check freshness after acquiring the rotation lock, so callers that queued behind a rotation adopt its result instead of rotating again.
- Hand out a token only when it has at least 60 seconds of life left, so a leased token outlives the operation it was leased for.
- Rotate in exactly one place.
  The request helpers previously refreshed an expiring token themselves, outside the rotation lock, and the rotated pair was discarded rather than persisted.
  That retired the stored refresh token server-side while the keychain kept the dead one, so the next rotation failed and signed the user out.
  The request path now sends the token it was handed and reports a rejection instead of refreshing.
- Write to the keychain only as part of a rotation, not per request.
- Retry a request once when Daylite answers `401`, forcing one rotation first.
  Expiry-driven refresh cannot anticipate a token revoked server-side, which is the case this covers.
  Track a generation on the token state so a burst of rejected requests costs one rotation rather than one per caller.
- Do not retry an operation that has already mutated the local store.
  A rejected token surfaces there as an error the caller can retry deliberately.

## Amendment 2026-10-16: no stranded refresh tokens

Daylite retires the previous refresh token as soon as it issues a new one, so any rotation whose result does not reach the keychain leaves a dead refresh token behind and signs the user out on the next start.
Two paths allowed that.

- A refused keychain write discarded the rotated pair from memory as well, so even the running session went on rotating with the dead token.
  The session now holds the rotated pair in memory before persisting it and marks it unpersisted until the write succeeds.
  While it is unpersisted, a lease retries the write under the rotation lock and fails with the keychain error instead of handing out the token, so the keychain heals as soon as access is granted.
- Exiting while a rotation was in flight, including the restart that installs an update, could drop the response after Daylite had already rotated.
  On `RunEvent::Exit` the app now marks the session as closing, waits, bounded by a timeout, for the rotation lock, and persists any pending pair.
  A closing session refuses to start a rotation or connect, including for callers that were already queued for the lock, so a failed in-flight rotation cannot hand the lock to a fresh one while the exit timeout runs out.

Connecting saves the Daylite base URL after the refresh token is minted and before the session adopts the new pair.
A pair kept in memory after a refused keychain write therefore always matches the stored base URL.

A crash or power loss between Daylite's response and the keychain write still loses the new pair, because the server-side rotation cannot be made atomic with the local write.

## Consequences

- Daylite calls are typed and centrally error-normalized.
- Access token refresh is deterministic and based on expiry, with a `401` retry as the fallback for server-side revocation.
- Token rotation survives app restarts because the rotated token state, including expiry, is persisted in the keychain.
- Concurrent Daylite reads no longer queue behind one another, and keychain access is proportional to rotations rather than to requests.
- The in-memory token state is authoritative for the running process, so a keychain edit made elsewhere is not observed until the next start.
