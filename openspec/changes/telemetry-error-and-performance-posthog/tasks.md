## 1. Local store telemetry settings

- [ ] 1.1 Write failing `cargo test`s in `src-tauri/src/integrations/local_store/types.rs`: telemetry defaults to disabled, a store file without a `telemetry` section loads without error, and the install identifier round-trips through save and reload
- [ ] 1.2 Add `TelemetrySettings { enabled: bool, install_id: Option<String> }` to `LocalStore` behind `#[serde(default)]`, satisfying the tests
- [ ] 1.3 Run `cargo test` in `src-tauri` and confirm the existing local store tests still pass

## 2. Telemetry module foundation

- [ ] 2.1 Create `src-tauri/src/integrations/telemetry/mod.rs` and register it in `integrations/mod.rs`
- [ ] 2.2 Write failing tests for the `Operation` and `Integration` enums serializing to stable snake_case dimension values
- [ ] 2.3 Implement `events.rs` with the `Operation` and `Integration` enums and the `operation_completed` / `error_occurred` / `app_started` event structs, satisfying the tests
- [ ] 2.4 Write failing tests for the redaction sanitizer: a message containing a URL, an absolute file path, a bearer token, and an e-mail address comes out with each value redacted and the surrounding description retained
- [ ] 2.5 Implement `redact.rs` and make the event constructors the only way to build an event, running every free-text field through the sanitizer
- [ ] 2.6 Write a failing test asserting that an event built from a `ZepError` whose technical message embeds a calendar URL transmits no URL
- [ ] 2.7 Satisfy that test and confirm structured dimensions carry no free text

## 3. Consent gate and install identity

- [ ] 3.1 Write failing tests: no event is recorded while telemetry is disabled, events are recorded once enabled, and disabling discards pending events
- [ ] 3.2 Implement the consent gate in the telemetry recorder, reading the flag from the local store, satisfying the tests
- [ ] 3.3 Write failing tests: an install identifier is generated on first activation, is stable across reloads, and is a random `uuid` v4 rather than derived from user or host data
- [ ] 3.4 Implement install identifier generation and persistence, satisfying the tests

## 4. Buffering and delivery

- [ ] 4.1 Write failing tests for the bounded buffer: events flush at twenty buffered events, flush on the interval tick, drop oldest-first at the five hundred event cap, and recording returns without awaiting delivery
- [ ] 4.2 Implement `queue.rs` with the bounded `tokio::sync::mpsc` channel and the background flush task, satisfying the tests
- [ ] 4.3 Write a failing test asserting the client is inert when `POSTHOG_API_KEY` is absent at compile time (no request attempted)
- [ ] 4.4 Implement `client.rs` posting batches to the EU `/batch/` endpoint with the `option_env!` key, satisfying the test
- [ ] 4.5 Add a cassette-based test for the capture request using the existing `http_record_replay` harness, asserting the batch payload shape and that `distinct_id` is the install identifier
- [ ] 4.6 Write a failing test asserting a failed delivery attempt is not surfaced to the caller and the buffer stays bounded
- [ ] 4.7 Implement drop-after-one-retry delivery failure handling, satisfying the test

## 5. Event context

- [ ] 5.1 Write failing tests asserting every transmitted event carries the app version, the OS name and version, and the install identifier
- [ ] 5.2 Implement context enrichment applied at batch build time, satisfying the tests

## 6. Tauri wiring

- [ ] 6.1 Add `telemetry_get_settings` and `telemetry_set_enabled` commands and register them in the specta builder in `src-tauri/src/lib.rs`
- [ ] 6.2 Add the `telemetry_capture_frontend_error` command accepting error name, message, and context, routing through the same redaction and consent gate
- [ ] 6.3 Initialize telemetry state and spawn the flush task in the Tauri `setup` hook
- [ ] 6.4 Emit the `app_started` event with the startup duration measured from process start to the ready state
- [ ] 6.5 Add a bounded two second final flush on `RunEvent::Exit`
- [ ] 6.6 Run `cargo test` to regenerate `src/generated/tauri.ts` via the bindings test and confirm the new commands appear

## 7. Consent UI

- [ ] 7.1 Write failing tests in `src/app/components/settings/telemetry-panel.spec.tsx`: the toggle reflects the stored state, toggling persists via the command, and the German description names what is and is not transmitted
- [ ] 7.2 Implement `telemetry-panel.tsx` with the DaisyUI toggle and the German description, satisfying the tests
- [ ] 7.3 Add the "Diagnose" section to `sections` in `settings-dialog.tsx` and render the panel
- [ ] 7.4 Update `settings-dialog` tests for the new section

## 8. Frontend error capture

- [ ] 8.1 Write failing tests for the root error boundary: a child render error reports through the telemetry command and a German fallback message is displayed
- [ ] 8.2 Implement the error boundary and mount it at the application root in `src/app/page.tsx`
- [ ] 8.3 Write failing tests for the global `error` and `unhandledrejection` handlers forwarding to the telemetry command
- [ ] 8.4 Implement the global handlers, satisfying the tests
- [ ] 8.5 Verify no frontend module references the PostHog endpoint or key

## 9. Backend instrumentation

- [ ] 9.1 Implement the `telemetry::observe(Operation, future)` helper with tests covering success, failure, and that the measured duration is emitted in both cases
- [ ] 9.2 Wrap the calendar commands in `calendar/commands.rs` (`load_week_events`, `create_assignment`, `update_assignment`, `move_assignment`, `reorder_assignment`, `delete_assignment`)
- [ ] 9.3 Instrument the CalDAV request sites in `calendar/caldav/`, replacing the `eprintln!` failure logs in `write.rs` and `protection.rs` with telemetry events plus the existing log where it still aids local debugging
- [ ] 9.4 Instrument the Daylite request sites in `daylite/client.rs`, `projects.rs`, `categories.rs`, and `contacts/`, ensuring no search term or project name enters an event
- [ ] 9.5 Instrument the ZEP commands and credential loading in `zep/`, ensuring no calendar URL or password enters an event
- [ ] 9.6 Instrument the holiday API request in `holidays.rs`, replacing its four `eprintln!` failure paths
- [ ] 9.7 Instrument local store read and write failures with the existing `StoreErrorCode` as the error dimension

## 10. Build configuration

- [ ] 10.1 Wire `POSTHOG_API_KEY` into the release build workflow only, leaving development and CI builds without it
- [ ] 10.2 Confirm a build without the variable produces an inert client and a passing test suite

## 11. Documentation

- [ ] 11.1 Write `docs/adr/0013-telemetry-and-posthog-integration.md` recording the opt-in default, the backend-only client, the EU region choice, and the data minimization rules
- [ ] 11.2 Add telemetry to the README implemented-features list
- [ ] 11.3 Run `bun run test:docs`

## 12. Verification

- [ ] 12.1 Run `cargo test` in `src-tauri` and confirm all tests pass
- [ ] 12.2 Run `bun test` and confirm all tests pass
- [ ] 12.3 Run `bun lint` and fix any issues
- [ ] 12.4 Manually verify with telemetry disabled that no outbound request reaches the telemetry endpoint
- [ ] 12.5 Manually verify with telemetry enabled against a scratch PostHog project that a forced Daylite failure and a week load appear as `error_occurred` and `operation_completed` events carrying no personal or business data
- [ ] 12.6 Run `bunx openspec validate telemetry-error-and-performance-posthog --strict`
