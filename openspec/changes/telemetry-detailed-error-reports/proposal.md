## Why

Telemetry error events are too heavily redacted to diagnose a user's problem remotely.
A Daylite timeout arrives as "Zeitüberschreitung bei [redacted]: error sending request for url ([redacted])", which names neither the request nor the server.
Telemetry is opt-in, so the maintainer accepts sending full error details from users who agreed to it, provided the consent text says so and credentials never leave the machine.

The current sanitizer also misses credentials embedded in JSON, so a Daylite token refresh answered with `expires_in=0` would transmit both Daylite tokens.

## What Changes

- Redact only credentials from free-text event fields: bearer and basic authorization values, credential key-value pairs in plain text, URL query strings and JSON, and user credentials embedded in a URL.
- Keep URLs, file paths, e-mail addresses and all other error text.
- Transmit the message of commands that fail with only a user-facing German message, instead of only `COMMAND_FAILED`.
- Restore the resource URL, the target calendar URL and the project reference at the CalDAV and project lookup failure sites.
- Keep the client IP address that PostHog records.
- Rewrite the German consent text to state that error reports contain technical details, can contain project or employee names and include the IP address, and that passwords, keys and tokens are never transmitted.
- **BREAKING** for the telemetry spec: the data minimization requirement is replaced by a secret removal requirement.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `telemetry`: data minimization is replaced by secret removal, and the consent text discloses the detailed error data and the IP address.

## Impact

- `src-tauri/src/integrations/telemetry/redact.rs`: new redaction rules.
- `src-tauri/src/integrations/telemetry/observe.rs`: `String` command errors carry their message.
- `src-tauri/src/integrations/calendar/caldav/write.rs` and `src-tauri/src/integrations/calendar/protection.rs`: failure details restored.
- `src/app/components/settings/telemetry-panel.tsx`: German consent text.
- `docs/adr/0014-detailed-telemetry-error-reports.md`: records the reversal of the data minimization rule from ADR 0013.
- `openspec/specs/telemetry/spec.md`: Purpose updated.
- No release has shipped telemetry yet, so no installed client consented under the previous text.
