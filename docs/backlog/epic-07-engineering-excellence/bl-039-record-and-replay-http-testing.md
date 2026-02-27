# BL-039: Implement Secure Record & Replay HTTP Testing Infrastructure

## Scope

- Implement a VCR-style testing architecture in the Rust backend to mock external API interactions without live network calls.
- Integrate the reqwest-vcr (or equivalent middleware) into the existing reqwest::Client builder used by the Tauri backend.
- Create a configuration toggle allowing tests to run in either Record mode (against the live external API) or Replay mode (using local files).
- Implement a pre-sanitization hook within the middleware pipeline to strip Authorization headers before they are written to disk.
- Setup git-crypt for the tests/cassettes/ directory so that intercepted payload data (which may contain sensitive data) is encrypted at rest in the repository.

## Acceptance Criteria

- Running tests with an environment variable (e.g., VCR_MODE=record) successfully makes live network requests and saves the HTTP request/response to a .json or .yaml cassette file in tests/cassettes/.
- Running tests normally (e.g., cargo test) replays the recorded cassettes without making any external HTTP requests, and the tests pass deterministically.
- Cassette files generated during the Record mode NEVER contain Authorization, Cookie, or x-api-key headers (headers are scrubbed/omitted).
- Any files inside tests/cassettes/ are automatically encrypted when committed and pushed to git (verified by checking the raw git history).
- CI/CD pipeline is updated with the git-crypt unlock key (stored as a CI secret) to successfully run the test suite using the decrypted cassettes.

## Dependencies

- Requires the reqwest-middleware and reqwest-vcr (or rvcr) crates added to Cargo.toml as [dev-dependencies].
- Requires git-crypt to be installed on developer machines and the CI/CD runner.
- Depends on the existing external API client modules.

## Out of Scope

- Scrubbing deeply nested JSON payloads (we rely on git-crypt to protect the body data; only headers are scrubbed).
- Rewriting existing unit tests that already use manually written local mocks (this ticket is for establishing the infrastructure and migrating the core integration tests).
- Generating sandbox/synthetic data on the external API.

## Implementation Notes

- VCR Setup: Wrap the reqwest::Client using reqwest_middleware::ClientBuilder and reqwest_vcr::VCRMiddleware. Set the middleware mode based on the VCR_MODE environment variable.
- Header Scrubbing: Use the middleware filtering capabilities (or a custom middleware layer executing before the VCR middleware) to strip sensitive headers.
- Git-Crypt Setup: Add .gitattributes file targeting tests/cassettes/** filter=git-crypt diff=git-crypt. Initialize git-crypt and export the symmetric key.
- CI/CD: Update the GitHub Actions/GitLab CI workflow to install git-crypt, echo the base64-encoded secret key to a file, and run git-crypt unlock before executing cargo test.

## Tests (write first)

- Sanitization Test: Write a dummy test in Record mode using a mock HTTP server. Assert that the generated cassette file on disk does not contain the injected Authorization header.
- Replay Determinism Test: Write a test that runs twice in Replay mode. Assert that it returns the exact same data without making a network call (assert network latency is near 0ms).
- Crypto Configuration Test: Add a bash script to the CI pipeline to cat a cassette file before git-crypt unlock is called, asserting that the file is binary/encrypted and not readable JSON.
