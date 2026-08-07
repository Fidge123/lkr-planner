## Why

The application currently reports failures only through `eprintln!` in the Rust backend and German error strings in the UI, so nobody outside the running machine ever learns that a Daylite refresh failed, a CalDAV write was rejected, or a week load took twelve seconds.
Because this is a distributed desktop app, there is no server log to inspect after a user reports "es hängt" — the evidence is gone by the time the report arrives.
Sending structured error and duration events to PostHog (free tier) makes recurring integration failures and slow operations visible per release without a support ticket.

## What Changes

- Add a `telemetry` capability in the Rust backend that buffers events and sends them to the PostHog capture API in batches, with the project API key baked in at build time.
- Capture error events for failed Tauri commands and failed outbound integration calls (Daylite, ZEP/CalDAV, Nager holidays, local store, keychain), carrying the existing error `code`, the integration name, and the technical message.
- Capture error events for uncaught frontend errors and unhandled promise rejections via a React error boundary and global handlers.
- Capture performance events measuring the duration of Tauri commands, outbound HTTP calls to each integration, and app startup.
- Attach anonymous context to every event: app version, OS and version, and a locally generated random install ID persisted in the local store.
- Add an opt-in telemetry consent flag, off by default, exposed as a new "Diagnose" section in the settings dialog; no event is buffered or sent while it is off.
- Never include personal or business data in events: no project names, contact names, calendar URLs, tokens, or free-text search input — only enumerated codes, integration names, durations, and sanitized technical messages.
- Fail silently and never block the user: telemetry delivery errors are dropped, the buffer is bounded, and no UI depends on a capture result.

## Capabilities

### New Capabilities

- `telemetry`: Opt-in collection and delivery of error and performance events to PostHog, covering the consent gate, event taxonomy and payload shape, anonymous install identity, batching and delivery behavior, data minimization rules, and failure isolation.

### Modified Capabilities

- `local-store`: The persisted store gains a telemetry section holding the opt-in flag and the anonymous install ID, so consent and identity survive restarts.

## Impact

- `src-tauri/src/integrations/telemetry/`: new module with the PostHog client, event queue, background flush task, event types, and the redaction rules.
- `src-tauri/src/lib.rs`: register telemetry state and the new Tauri commands, start the flush task in `setup`, and record app startup duration.
- `src-tauri/src/integrations/local_store/types.rs`: new `TelemetrySettings` field (`enabled`, `installId`) with a default that keeps telemetry off for existing stores.
- `src-tauri/src/integrations/{daylite,calendar,zep,holidays}`: emit error and duration events at the existing failure and request sites that today only `eprintln!`.
- `src/app/components/settings/`: new `telemetry-panel.tsx` plus a "Diagnose" entry in `settings-dialog.tsx`.
- `src/app/`: new error boundary and global `error`/`unhandledrejection` handlers forwarding to the backend.
- `src/generated/tauri.ts`: regenerated specta bindings for the new commands.
- Build/CI: a `POSTHOG_API_KEY` build-time environment variable; when it is absent the telemetry client is inert, so local dev and tests send nothing.
- Tests: Rust unit tests for consent gating, redaction, batching, and bounded buffering; cassette-based tests for the capture request; frontend tests for the consent panel and error boundary.
- `docs/adr/0013-telemetry-and-posthog-integration.md`: new ADR recording the opt-in model, the backend-only client, and the data-minimization rules.
