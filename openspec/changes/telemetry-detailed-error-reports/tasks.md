## 1. Secret removal

- [x] 1.1 Rewrite the tests in `src-tauri/src/integrations/telemetry/redact.rs` for the new rules: URLs, paths and e-mail addresses retained; credentials removed from plain-text pairs, URL query strings, JSON with and without spaces, truncated JSON, a URL's authority, and bearer values; words that merely end in a credential name retained
- [x] 1.2 Implement the new rules in `redact.rs`, satisfying the tests
- [x] 1.3 Update the event and observe tests that asserted URLs and paths were removed

## 2. Error detail

- [x] 2.1 Write a failing test that a command failing with a `String` error transmits its message
- [x] 2.2 Transmit the message for `String` command errors in `observe.rs`
- [x] 2.3 Restore the resource URL, the target calendar URL and the project reference at the CalDAV and project lookup failure sites

## 3. Consent text

- [x] 3.1 Update `src/app/components/settings/telemetry-panel.spec.tsx` for the new German text
- [x] 3.2 Rewrite the German consent text in `telemetry-panel.tsx`, satisfying the tests

## 4. Documentation

- [x] 4.1 Write ADR 0014 and mark the data minimization decision in ADR 0013 as superseded
- [x] 4.2 Update the Purpose of the main telemetry spec and the README feature entry
- [x] 4.3 Run `bun run test:docs`

## 5. Verification

- [x] 5.1 Run `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` in `src-tauri`
- [x] 5.2 Run `bun test`, `bun run build` and `bunx biome ci .`
- [x] 5.3 Run `bunx openspec validate telemetry-detailed-error-reports --strict`
