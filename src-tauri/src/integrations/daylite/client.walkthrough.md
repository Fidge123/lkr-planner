# Walkthrough: `src-tauri/src/integrations/daylite/client.rs`

## Purpose

This file is the HTTP transport layer for Daylite. It wraps `reqwest`, defines a small transport trait for testability, and integrates record/replay cassettes for integration-style tests.

## Block by block

### Imports (`lines 1-12`)

- Production code uses `normalize_base_url`, the shared error model, `serde_json::Value`, `Arc`, and `reqwest`.
- Test-only imports pull in the cassette record/replay types behind `#[cfg(test)]`.

Rust syntax to notice:
- `#[cfg(test)] use ...;` removes test-only dependencies from release builds.

### `DayliteApiClient` wrapper (`lines 13-84`)

- The client stores `transport: Arc<dyn DayliteHttpTransport>`.
- `new` builds the default `ReqwestTransport`.
- `with_transport` allows injecting mocks in tests.
- `with_replay_cassette`, `with_env_cassette`, and `with_cassette` are convenience constructors for test scenarios.
- `send_request` simply packages request data into a `DayliteHttpRequest` and delegates to the transport.

Rust syntax to notice:
- `dyn DayliteHttpTransport` is a trait object: the concrete transport can vary at runtime.
- `Arc` gives cheap shared ownership across async code and tests.

Best practice:
- Wrap the concrete HTTP client behind a small interface so business logic stays testable.

### Async trait workaround types (`lines 86-115`)

- `type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;` is a boxed future alias.
- `DayliteHttpTransport` uses that alias because stable Rust traits cannot yet express async methods directly without extra machinery.
- `DayliteHttpMethod`, `DayliteHttpRequest`, and `DayliteHttpResponse` make the transport boundary explicit.

Rust syntax to notice:
- `Pin<Box<...>>` is common when returning dynamically-dispatched futures.
- Small request/response structs are often easier to test than passing raw `reqwest` types around.

### Concrete `ReqwestTransport` (`lines 117-155`)

- `ReqwestTransport` stores the normalized base URL, a `reqwest::Client`, and, in tests, an optional cassette config.
- `new` validates the base URL and constructs the HTTP client.
- `new_with_record_replay` reuses `new` and then injects the cassette mode.

Best practice:
- Normalize the base URL once at construction time instead of revalidating it for every request.

### `send` implementation (`lines 157-284`)

- In test builds, the function first derives a cassette-friendly `RecordedRequest`.
- If record/replay is enabled in replay mode, it attempts to return a matching recorded response before touching the network.
- Otherwise it builds the full request URL, appends query pairs, chooses the HTTP verb, conditionally adds the `Authorization` header, and serializes any JSON body.
- After the network call, it reads the status and body text.
- In test builds with record mode enabled, it writes the interaction back to the cassette.
- Finally it returns `DayliteHttpResponse`.

Rust syntax to notice:
- `Box::pin(async move { ... })` turns the async block into the boxed future required by the trait.
- `match request.method { ... }` selects the correct `reqwest` builder.
- `if let Some(access_token) = request.access_token { ... }` handles optional auth cleanly.

Best practice:
- Keep transport concerns here: URL assembly, headers, body serialization, and cassette plumbing should not leak into higher-level modules.

### Test-only helpers (`lines 286-312`)

- `DayliteHttpMethod::as_str` converts enum variants to stable cassette keys.
- `to_recorded_request` deliberately excludes sensitive headers so cassettes stay sanitized.
- `cassette_path_for_test` keeps all cassette files under `tests/cassettes`.

### Tests (`lines 314-423`)

- The tests verify cassette recording, replay without real network access, and `VCR_MODE` handling.
- `env_lock` uses `OnceLock<Mutex<()>>` because environment variables are process-global and tests may otherwise race.

Rust syntax to notice:
- `OnceLock` initializes shared state exactly once.
- `unsafe { std::env::set_var(...) }` and `remove_var` appear because mutating environment variables is `unsafe` in Rust 2024 due to process-wide race concerns.

## Best practices this file demonstrates

- Hide concrete HTTP libraries behind a domain-specific transport trait.
- Keep test recording and replay logic close to the transport boundary.
- Sanitize what gets persisted into cassettes.
