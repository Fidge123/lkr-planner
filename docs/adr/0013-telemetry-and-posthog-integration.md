# ADR 0013: Telemetry and PostHog Integration

- Status: Accepted
- Date: 2026-08-15

## Context

The application reported failures only through `eprintln!` in the Rust backend and German error strings in the UI.
A bundled desktop application writes stderr nowhere the user can see, so in practice a shipped build had no observability at all.
When a planner reports "es hängt", the evidence is already gone: there is no server log to inspect and no record of which integration failed or how long an operation took.

The application is distributed to user machines, processes German business data, and integrates with Daylite, ZEP via CalDAV, and the Nager holiday API.
Errors already travel as structured payloads (`StoreError`, `ZepError`, `DayliteApiError`) carrying `code`, `user_message`, and `technical_message`, so an enumerated error dimension exists without inventing one.

### Evaluated Options
- Send opt-in error and performance events to PostHog from the Rust backend
  - Pros: Aggregates failures and durations per release across installs without a support ticket; free tier covers the expected volume; the existing error codes become ready-made dimensions; the backend already owns all third-party API logic.
  - Cons: Requires a consent gate whose text discloses the transmitted data and rules that keep credentials out of every event; opt-in means sparse coverage; adds a build-time key to the release pipeline.
- Send the same events but redact URLs, paths, e-mail addresses, and command messages as well as credentials
  - Pros: No personal or business data leaves the machine.
  - Cons: A report like "Zeitüberschreitung bei [redacted]" names neither the failing request nor the server, so diagnosing a user's problem still needs the user.
- Write a local rotating log file with a "Diagnose exportieren" button
  - Pros: No consent question because data never leaves the machine; answers "what happened on this machine ten minutes ago" directly; far less work.
  - Cons: Requires the user to notice, export, and send the file, so recurring failures across installs stay invisible; no aggregate view per release.
- Use a frontend PostHog SDK instead of a backend client
  - Pros: Least backend code; autocapture and session replay available out of the box.
  - Cons: Violates the project rule that all third-party API logic lives in the Rust backend; the frontend would hold the project key; backend failures that never reach the UI would go unrecorded.

## Decision

- Add a `telemetry` capability in the Rust backend that buffers events and posts them in batches to the PostHog EU region endpoint (`https://eu.i.posthog.com/batch/`), so event data stays in the EU.
- Default telemetry to off and require explicit consent, persisted in the local store as `telemetry.enabled` behind `#[serde(default)]`, surfaced as a "Diagnose" section in the settings dialog.
- Name PostHog with EU servers as the recipient in the German consent text, and state that error reports contain technical details that can include project or employee names and the IP address that PostHog records, and that passwords, keys, and tokens are never transmitted.
- Identify events by a random UUID v4 install identifier persisted in the local store, never derived from user, host, or hardware data.
- Read the project key from `option_env!("POSTHOG_API_KEY")` at compile time, set only in the release workflow, so development and CI builds compile to an inert client that sends nothing.
- Carry everything in three event names: `operation_completed` (operation, integration, success, duration), `error_occurred` (operation, integration, code, message), and `app_started` (startup duration).
- Keep the full error detail, including URLs, file paths, e-mail addresses, and the messages of commands that fail with only a German user message, so a user's problem can be diagnosed remotely.
- Remove credentials structurally: event constructors are the only way to build an event and run every free-text field through a sanitizer that redacts bearer and basic values, user credentials in a URL, and values whose key ends in token, password, passwort, passwd, secret, key, apikey, authorization, or auth, in plain text, URL query strings, and JSON.
  Credentials are recognised by key rather than by shape, because the Daylite refresh token travels as a URL query parameter that HTTP errors repeat and token refresh errors embed the response body as JSON, while matching long random strings would miss short secrets and redact IDs that debugging needs.
- Instrument through one `observe` helper wrapping each Tauri command and the outbound request paths, so a call site is annotated once and emits both its duration and its failure.
- Isolate failures: recording is non-blocking, the channel drops events when full, the buffer caps at 500 events dropping oldest-first, a failed batch is retried once and then dropped, and shutdown waits at most two seconds for a final flush.

## Consequences

- Recurring integration failures and slow operations become visible per release without a support ticket, for the installs that opted in, and a single user's error report is precise enough to diagnose remotely.
- Error reports are personal data, since they can contain project names, e-mail addresses, and the IP address, and they are collected only after opt-in under a text that names these categories.
- A credential stored under a key name outside the list would be transmitted, so a new credential needs its key name added.
- Opt-in defaults mean most installs report nothing, so the data may be too sparse to spot a regression.
  Moving to opt-out would produce far better coverage but needs a consent notice and a documented legal basis, which is a larger decision than this change made.
- The PostHog project key is extractable from the shipped binary.
  This is accepted: a project API key is a write-only ingestion key that grants no read access, and a leaked key is recoverable by rotating it.
- A hang produces neither a completion nor an error event, so the symptom that motivated this change is not itself observable through the current event taxonomy.
  Making it visible needs an in-flight signal, which a later change can add.
- `os_info` was added as a dependency because the standard library exposes no operating system version.
- Telemetry adds an event per command invocation, which is well inside the PostHog free tier at the expected volume, provided nothing per-render, per-keystroke, or per-drag-frame is ever instrumented.
