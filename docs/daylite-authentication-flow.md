# Daylite Authentication Flow

This document describes how authentication works in LKR Planner for Daylite API calls.

## Endpoint

- Refresh endpoint: `GET /personal_token/refresh_token?refresh_token={refreshToken}`
- Expected response shape:

```json
{
  "access_token": "<NEW_ACCESS_TOKEN>",
  "expires_in": 3600,
  "token_type": "Bearer",
  "scope": "daylite:read daylite:write",
  "refresh_token": "<NEW_REFRESH_TOKEN>"
}
```

Only these token keys are accepted:
- `access_token`
- `refresh_token`
- `expires_in`

## Stored Token State

The app stores:
- `dayliteAccessToken`
- `dayliteRefreshToken`
- `dayliteAccessTokenExpiresAtMs`

`dayliteAccessTokenExpiresAtMs` is calculated from `expires_in` when a new access token is issued.

## Request Lifecycle

Before a Daylite API request, the app checks access-token expiry:
- If access token is missing, refresh immediately.
- If access token is expired, refresh immediately.
- If access token expires in less than 10 seconds, refresh immediately.
- Otherwise, use current access token.

After refresh:
- The new `access_token` is used for API requests.
- The new `refresh_token` replaces the old one and is persisted.
- Refresh tokens are treated as single-use and are never reused after successful rotation.

## Error Handling

- Missing access + refresh token: authentication error.
- Invalid refresh response payload: token refresh error.
- Downstream Daylite `401` responses are surfaced as unauthorized errors.
