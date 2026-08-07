## Context

See proposal.md - Why for the motivation.

Constraints that shape the approach:

- CLAUDE.md requires all third-party API logic to live in the Rust backend, so the PostHog client cannot be a frontend SDK; the frontend reports errors through a Tauri command.
- Errors already travel as structured payloads (`StoreError`, `ZepError`, and siblings) carrying `code`, `user_message`, and `technical_message`, so an event taxonomy can reuse `code` as an enumerated dimension without inventing one.
- Failure sites today are `eprintln!` calls in `daylite/projects.rs`, `calendar/caldav/write.rs`, `calendar/protection.rs`, `holidays.rs`, and `lib.rs`; these are the natural instrumentation points.
- `uuid` (v4), `reqwest` (via `tauri-plugin-http`), `tokio`, `serde`, and `chrono` are already dependencies, so no new crate is required.
- The repository has a record/replay HTTP harness (`integrations/http_record_replay.rs`) used for integration tests, which the capture request can reuse.
- PostHog's free tier allows one million events per month, which bounds how finely operations may be instrumented.
- The application and its users are German, so the EU-hosted PostHog region is the appropriate target.

## Goals / Non-Goals

**Goals:**

- One instrumentation helper that wraps an operation and emits both its duration and its failure, so a call site is annotated once rather than twice.
- A single redaction boundary that every outbound event passes through, so data minimization is enforced structurally instead of by reviewer discipline.
- Telemetry that is inert by construction in tests, local development, and builds without a project key.

**Non-Goals:**

- No session replay, autocapture, feature flags, or PostHog product analytics features beyond event capture.
- No crash reporting for hard process aborts (a panic that kills the process is out of reach for an in-process buffer); only recoverable errors and caught panics are covered.
- No local log file, log viewer, or in-app diagnostics export.
- No per-user identification or PostHog person profiles beyond the anonymous install identifier.

## Decisions

### Opt-in consent, stored in the local store

Telemetry defaults to off and requires an explicit user action to enable, with the flag persisted in `LocalStore` under a `telemetry` section behind `#[serde(default)]`.

This is an assumption made without confirmation from the user, chosen because the app processes German business data and an opt-in default is the position that needs no further legal argument.
The alternative, opt-out with a first-run notice, produces far better coverage and remains defensible under legitimate interest, but it requires a consent notice and a documented basis, which is a larger surface than this change should decide unilaterally.
Switching later is a one-line default change plus the notice, so the cheap direction is to start opt-in.

The flag lives in the local store rather than the keychain because it is a preference, not a secret.

### Backend-only PostHog client with a build-time key

A `telemetry` module in `src-tauri/src/integrations/` owns the only code that knows the PostHog endpoint, reading the project key from `option_env!("POSTHOG_API_KEY")` at compile time.
When the variable is absent the module compiles to an inert client that records nothing, which makes local development and the test suite silent without a runtime guard at every call site.

Bundling the key in the binary is acceptable because a PostHog project API key is a write-only ingestion key intended for client distribution; it grants no read access to captured data.

The target is the EU region endpoint (`https://eu.i.posthog.com/batch/`) so that event data stays in the EU.
The alternative, the US region default, would add a transfer question that the EU region simply avoids.

### One `observe` helper at the operation boundary

Instrumentation is a single async helper that takes an operation name and a future, measures elapsed time, inspects the result, and emits a performance event plus, on failure, an error event.

```
telemetry::observe(Operation::LoadWeekEvents, async { ... }).await
```

Each Tauri command body is wrapped once, and outbound request helpers per integration are wrapped once.
The alternative, a procedural macro over the specta command list, would instrument every command automatically but hides control flow and cannot express the integration-level request events, which are the more diagnostic half.
Sprinkling separate `record_duration` and `record_error` calls was rejected because the two drift apart the moment a call site gains an early return.

`Operation` is an enum, not a string, so the event dimension stays a closed set and a typo cannot create a new one in PostHog.

### Redaction as a constructor invariant

Event payloads are built only through constructors that run every free-text field through a sanitizer stripping URLs, file paths, bearer tokens, and e-mail addresses before the value is stored on the event.
The raw `technical_message` never reaches an event field unsanitized, so a new call site cannot leak by forgetting to sanitize.

Structured dimensions (integration, operation, error code, HTTP status, duration) are typed and carry no free text at all.
The alternative, sanitizing at serialization time, was rejected because it invites a future "just this once" field that bypasses the serializer path.

### Bounded queue with a background flush task

Recording sends the event over a bounded `tokio::sync::mpsc` channel and returns immediately; a background task owns the buffer and flushes on whichever comes first, a thirty second interval or twenty buffered events.
When the channel is full the event is dropped at the sender, which keeps a telemetry stall from applying back-pressure to a user-facing command.
The buffer caps at five hundred events and drops oldest-first, bounding memory during a long offline stretch.

A failed batch is retried on the next flush tick and then dropped rather than retried with backoff, because stale duration measurements have little diagnostic value and a persistent retry queue is more machinery than the problem warrants.

On `RunEvent::Exit` the task gets a bounded window of two seconds to send what it holds, so shutdown is not visibly delayed.

### Event volume within the free tier

Two event names carry everything: `operation_completed` (with `success`, `duration_ms`, `operation`, `integration`) and `error_occurred` (with `operation`, `integration`, `code`, `message`), plus `app_started` once per launch.

The estimate that keeps this inside the free tier: a planner performing roughly five hundred instrumented operations per working day, across ten installs, produces about one hundred thousand events per month, an order of magnitude under the one million limit.
Nothing per-render, per-keystroke, or per-drag-frame is instrumented, which is what would break that estimate.
Reporting both success and failure durations under one event name, rather than separate names per operation, keeps PostHog insights groupable by dimension instead of requiring a new insight per operation.

### Frontend errors travel through a command

A React error boundary at the application root plus `window.addEventListener` handlers for `error` and `unhandledrejection` forward to a `telemetry_capture_frontend_error` Tauri command.
The frontend therefore never holds the PostHog key or endpoint, satisfying the CLAUDE.md rule that third-party API logic stays in the backend, and frontend events pass through the same redaction and consent gate as backend events.

The error boundary renders a German fallback message, so an uncaught render error degrades to a readable screen rather than a blank one.

## Risks / Trade-offs

- [Opt-in default means most installs report nothing, so the data may be too sparse to spot a regression] → Accept for now; the settings copy explains the benefit, and the default can be revisited once the legal basis for opt-out is settled.
- [A new call site adds a free-text field that leaks business data] → Constructors sanitize on entry, and a unit test asserts that a technical message containing a URL, a path, and a token comes out redacted.
- [The PostHog key is extractable from the shipped binary] → Accepted; the key is write-only ingestion and cannot read captured data. A leaked key permits event spam into the project, which is recoverable by rotating it.
- [Instrumentation noise in command bodies obscures the domain logic] → One wrapping call per command, with the enum and helper in the telemetry module, keeps the diff at the boundary rather than inside the logic.
- [Duration measurements include cache hits and therefore mix two populations] → The performance event carries the operation name only; distinguishing a cached from an uncached load is left to a later change if the data proves ambiguous.
- [A misconfigured build could point telemetry at the wrong project] → The key is absent by default and telemetry is inert without it, so the failure mode is silence rather than misdirected data.

## Migration Plan

- Ship the local store field first, behind `#[serde(default)]`, so existing store files load unchanged and report telemetry as disabled.
- Build the telemetry module with its tests before wiring any call site, following the red/green order CLAUDE.md requires.
- Wire the consent panel next, so the flag is reachable before any event can be produced.
- Instrument call sites last, one integration at a time, since each is independent and none changes observable behavior.
- Rollback is reverting the commits; no data migration is involved, and an unknown `telemetry` key in a store file written by a newer build is ignored by older builds through the same `serde(default)` handling.
- Set `POSTHOG_API_KEY` in the release build workflow only, leaving development and CI builds inert.

## Open Questions

- Which retention window and project settings to configure in the PostHog project itself; this is a dashboard-side setting that does not affect the specs, the client, or the task breakdown.
