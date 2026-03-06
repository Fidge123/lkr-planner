# Walkthrough: `src-tauri/src/integrations/http_record_replay.rs`

## Purpose

This file implements a lightweight record/replay cassette format for tests. It can store normalized HTTP interactions on disk and later replay them without making network calls.

## Block by block

### Imports (`lines 1-5`)

- `serde` and `serde_json::Value` serialize cassette files.
- `Display` and `Formatter` support a small custom error type.
- `fs`, `Path`, and `PathBuf` handle file persistence.

### `VcrMode` enum (`lines 7-20`)

- `VcrMode` has two states: `Record` and `Replay`.
- `from_env` reads `VCR_MODE` and treats any value other than `"record"` as replay mode.

Rust syntax to notice:
- `eq_ignore_ascii_case("record")` gives forgiving environment parsing without allocating a lowercase copy.

Best practice:
- Defaulting to replay is safer because it avoids accidental live network writes during tests.

### `RecordReplayConfig` and its methods (`lines 22-113`)

- The config stores the cassette path and the current mode.
- `new` and `from_env` are the two constructors.
- `mode` exposes the current `VcrMode`.
- `replay` reads the cassette and returns the first matching interaction for a request.
- `record` loads or creates a cassette, removes any existing interaction with the same request key, appends the new interaction, and writes the file back.
- `read_existing_cassette`, `read_cassette_or_default`, and `write_cassette` are the file-system helpers behind those public operations.

Rust syntax to notice:
- `.retain(...)` removes old matching entries in place.
- `Result<Option<RecordedResponse>, RecordReplayError>` means replay itself can fail, and even on success there may simply be no matching cassette entry.

Best practice:
- Deduplicating by request key keeps cassette files deterministic when the same endpoint is re-recorded.

### Recorded interaction types (`lines 115-148`)

- `RecordedRequest` captures only the stable request identity: method, path, query, and JSON body.
- `RecordedResponse` stores the status code and body text.
- `RecordedInteraction` pairs the two.
- `CassetteFile` is the on-disk root document with a `version` field for future format changes.

Best practice:
- Keep cassette keys stable and minimal; sensitive headers and transient transport details should not be part of the match key.

### Error wrapper (`lines 150-167`)

- `RecordReplayError` stores one formatted message string.
- `new` prepends the filesystem path to the error message.
- The `Display` implementation writes that message to any formatter.

Rust syntax to notice:
- Implementing `Display` is enough for readable error messages even without implementing the full `std::error::Error` trait.

## Best practices this file demonstrates

- Make test fixtures explicit, versioned, and human-readable.
- Separate cassette I/O from business-level HTTP logic.
- Include the path in filesystem errors to speed up debugging.
