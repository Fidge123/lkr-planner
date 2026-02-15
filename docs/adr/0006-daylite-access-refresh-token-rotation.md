# ADR 0006: Daylite Access/Refresh Token Rotation and Persistence

- Status: Accepted
- Date: 2026-02-14

## Context

BL-006 requires a minimal Daylite API client for project/contact read and search operations with normalized errors.
Daylite issues short-lived access tokens and single-use refresh tokens.
Without persistence of rotated tokens and access token expiry, authenticated requests will fail after restart or token rotation.

## Decision

- Implement the Daylite API client in Rust (`src-tauri/src/integrations/daylite.rs`) behind Tauri commands.
- Use the Daylite refresh endpoint (`/personal_token/refresh_token`) with query parameter `refresh_token`.
- Normalize all Daylite failures into one typed error payload containing:
  - machine-readable error code
  - optional HTTP status
  - German user-facing message
  - technical debug message
- Persist the latest rotated Daylite tokens in the local store:
  - `tokenReferences.dayliteAccessToken`
  - `tokenReferences.dayliteRefreshToken`
  - `tokenReferences.dayliteAccessTokenExpiresAtMs`
- On Daylite requests:
  - refresh access token only when it is expired or within 10 seconds of expiry
  - compute access-token expiry using `expires_in`
  - always replace and persist the refresh token returned by the refresh call
  - send the current access token for API requests

## Consequences

- Daylite calls are typed and centrally error-normalized.
- Access token refresh is deterministic and based on expiry instead of reactive `401` retries.
- Token rotation survives app restarts because token state (including expiry) is stored with the existing local store mechanism.
