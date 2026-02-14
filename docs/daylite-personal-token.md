# Daylite Personal Token Setup (End User)

This app uses the Daylite personal token flow described in the official docs:
- [Daylite Personal Token](https://developer.daylite.app/reference/personal-token)

## 1) Request a personal token in Daylite

1. Open Daylite and go to the personal token section (as documented on the Daylite page above).
2. Create/request a personal token.
3. Copy the token value immediately.

## 2) Connect the token in LKR Planner

1. Enter your Daylite base URL in the app.
2. Paste the personal token once.
3. Confirm the connection.

The app exchanges this personal token through `/personal_token/refresh_token` and stores the current Daylite access/refresh tokens locally.

## 3) Why you normally do this only once

- Daylite can rotate access and refresh tokens.
- LKR Planner keeps track of these changed tokens automatically after requests.
- The latest token pair is persisted in the local app store, so a restart does not require a new personal token request.

You only need to reconnect if Daylite invalidates your refresh token (for example after manual revocation).

## Security note

- Treat personal/access/refresh tokens as credentials.
- Do not share tokens in screenshots, chat, or tickets.
