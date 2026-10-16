# ADR 0014: Detailed Telemetry Error Reports

- Status: Accepted
- Date: 2026-10-16

## Context

ADR 0013 made data minimization a rule of the telemetry client: every free-text field passed a sanitizer that redacted URLs, absolute paths, e-mail addresses and credentials, and commands failing with a German user message sent no message at all.
In practice that removed exactly what diagnosing an error needs.
A Daylite timeout arrived as "Zeitüberschreitung bei [redacted]: error sending request for url ([redacted])", naming neither the request nor the server.
The sanitizer also matched only `key=value` pairs, so a Daylite token refresh answered with `expires_in=0` would have transmitted both Daylite tokens embedded as JSON.

Telemetry is opt-in, and it has not shipped in a release yet, so no client has consented under the earlier text.

### Evaluated Options
- Redact only credentials and send every other error detail, disclosed in the consent text
  - Pros: Error reports name the failing request, server and project, which makes remote diagnosis possible; the sanitizer remains the single boundary; credentials stay protected wherever they appear.
  - Cons: Reports contain personal and business data such as project names, e-mail addresses and IP addresses, which the consent text must disclose.
- Keep data minimization and add structured, non-identifying fields such as a request path template and the host
  - Pros: Reports stay free of personal and business data.
  - Cons: Every call site needs its own structured fields, and the free-text detail that explains most failures is still lost.
- Keep data minimization and ask users to send a local log on request
  - Pros: No personal data leaves the machine without an explicit action.
  - Cons: Needs a local log and an export path that do not exist, and depends on the user noticing and reporting the problem.

## Decision

- Redact only credentials: bearer and basic authorization values, values whose key ends in token, password, passwort, passwd, secret, key, apikey, authorization or auth in plain text, URL query strings and JSON, and user credentials in a URL's authority.
- Transmit URLs, file paths, e-mail addresses and all other error text unchanged.
- Transmit the message of commands that fail with only a German user message.
- Keep the IP address that PostHog records.
- State in the German consent text that error reports contain technical details that can include project or employee names, that the IP address is transmitted, and that passwords, keys and tokens never are.
- This supersedes the data minimization rules of ADR 0013, whose opt-in, EU region, backend-only client and delivery decisions remain.

## Consequences

- A user's error report is precise enough to diagnose remotely without a support call.
- Telemetry data is personal data, collected only after opt-in under a text that names its categories.
- A credential stored under a key name outside the list would be transmitted, so a new credential needs its key name added.
- A common word followed by a colon, such as "Passwort: falsch", is redacted, which loses a little text.
- The token leak in refresh error bodies is closed.
