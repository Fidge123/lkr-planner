## Context

ADR 0013 made data minimization a constructor invariant and redacted URLs, paths and e-mail addresses from every message.
In practice that removed exactly the details needed to diagnose an error: which request failed and against which server.
Telemetry is opt-in and has not shipped in a release yet.

## Goals / Non-Goals

**Goals:**

- Error reports precise enough to diagnose a user's problem without a support call.
- No credential is ever transmitted, however it appears in a message.
- A consent text that describes what is transmitted.

**Non-Goals:**

- No change to the event taxonomy, the batching, or the opt-in default.
- No re-consent mechanism, because no client has consented under the previous text.

## Decisions

### Redact credentials, keep everything else

The sanitizer stays the single boundary every free-text field passes through, but it now removes only values that grant access.
It recognises a credential by its key rather than its shape: a key whose last segment is token, password, passwort, secret, key, authorization or auth, followed by `=` or `:`, in plain text, URL query strings and JSON.
Recognising by key catches the Daylite refresh token, which travels as a `refresh_token` query parameter that the HTTP client's error text repeats, and the tokens inside a refresh response body.
Recognising secrets by shape, such as long random strings, was rejected because it misses short secrets and redacts IDs that debugging needs.

Bearer and basic authorization values and user credentials in a URL's authority are redacted as before.
A value without a terminator, such as a token cut off by log truncation, is redacted to the end of the message.

### Send command messages

Commands that fail with a German user message now transmit it, since it is the only description of those failures.

### Keep the IP address

PostHog records the sender's IP address by default.
It is kept to correlate reports from one location, and the consent text names it.

## Risks / Trade-offs

- [Error reports now contain personal and business data such as project names, e-mail addresses and IP addresses] → Collected only after opt-in under a text that names these categories.
- [A credential under a key name outside the list is transmitted] → The list covers every credential the app handles today, and a new credential needs its key added.
- [A common word followed by a colon, such as "Passwort: falsch", is redacted] → Accepted, since over-redaction loses little.

## Migration Plan

- Ship in the same release as the telemetry change, so no client sends data under the earlier consent text.

## Open Questions

- Whether a privacy policy outside the app must also name PostHog as a recipient.
