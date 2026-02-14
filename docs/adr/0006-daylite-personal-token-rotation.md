# ADR 0006: Daylite Personal Token Rotation and Persistence

- Status: Accepted
- Date: 2026-02-14

## Context

BL-006 requires a minimal Daylite API client for project/contact read and search operations with normalized errors.
The Daylite personal token flow also rotates access and refresh tokens over time.
Without persistence of rotated tokens, users would need to request and reconnect a personal token repeatedly.

## Decision

- Implement the Daylite API client in Rust (`src-tauri/src/integrations/daylite.rs`) behind Tauri commands.
- Use the Daylite personal token refresh endpoint (`/personal_token/refresh_token`) to exchange/refresh tokens.
- Normalize all Daylite failures into one typed error payload containing:
  - machine-readable error code
  - optional HTTP status
  - German user-facing message
  - technical debug message
- Persist the latest rotated Daylite tokens in the local store:
  - `tokenReferences.dayliteAccessToken`
  - `tokenReferences.dayliteRefreshToken`
- On Daylite requests:
  - send the stored access token
  - on `401`, refresh via stored refresh token and retry once
  - persist newly rotated tokens after successful calls

## Consequences

- Daylite calls are typed and centrally error-normalized.
- End users only need to request a personal token once in normal operation.
- Token rotation survives app restarts because token state is stored with the existing local store mechanism.
