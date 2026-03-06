# ADR 0010: HTTP Cassette Testing for Rust Integrations

- Status: Accepted
- Date: 2026-03-06

## Context

The Rust backend already centralizes external HTTP access in integration clients such as the Daylite transport.
Existing tests mostly stop at mocked transport boundaries, which leaves request assembly, response parsing, and deterministic replay of external API interactions unverified.
BL-039 requires record/replay HTTP testing with sanitized headers and encrypted cassette storage in the repository.

### Evaluated Options
- Add an off-the-shelf VCR middleware crate to the `reqwest` stack
  - Pros: Established pattern with ready-made cassette handling and request interception.
  - Cons: Requires a new dependency decision, adds abstraction we do not yet need, and makes header scrubbing behavior less explicit in our codebase.
- Implement a small in-house cassette layer at the Daylite transport seam
  - Pros: No new dependency approval needed, keeps cassette format simple JSON, and gives direct control over request matching and header scrubbing.
  - Cons: Covers only the current needs and lacks advanced VCR features such as richer matching rules or automatic middleware composition.
- Keep relying on manual mock transports only
  - Pros: Minimal implementation effort and no file-based cassette management.
  - Cons: Does not verify real HTTP request construction, cannot replay captured responses across test runs, and does not solve the sensitive-payload storage requirement.

## Decision

Implement a small JSON cassette layer in Rust and attach it to the Daylite `reqwest` transport for tests.
Recorded cassette entries store only method, path, query, body, status, and response body.
Sensitive headers are scrubbed by design because `Authorization`, `Cookie`, and `x-api-key` values are never serialized into cassette files.
Store checked-in cassettes under `tests/cassettes/` and protect that directory with `git-crypt` via repository attributes and CI unlock steps.

## Consequences

- Rust tests can replay checked-in HTTP responses deterministically without making network calls.
- Record mode remains available for environments where live HTTP access is allowed, while replay mode is the default safe path for CI and local test runs.
- The current cassette layer is intentionally narrow and may need extension before broader reuse across additional integrations such as Planradar.
- CI now depends on a `GIT_CRYPT_KEY_B64` secret when encrypted cassette blobs are present in the repository checkout.
