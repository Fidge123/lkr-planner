## Context

We need deterministic tests for external API integrations. Live network tests are slow, flaky, and require credentials. VCR-style recording solves this.

## Goals / Non-Goals

**Goals:**
- Record HTTP requests/responses to local files
- Replay recordings without network calls
- Sanitize auth headers from recordings
- Encrypt recordings at rest with git-crypt

**Non-Goals:**
- Deep JSON payload scrubbing (git-crypt handles this)
- Migrating existing unit tests to VCR
- Generating synthetic/sandbox API data

## Decisions

### VCR Library
**Decision**: Use reqwest-vcr (or rvcr) middleware
- Wraps reqwest::Client
- Supports record/replay modes
- Uses YAML or JSON cassette format

### Mode Configuration
**Decision**: Environment variable VCR_MODE controls behavior
- `record`: Makes live API calls, saves cassette
- `replay` (default): Uses local cassette, no network
- `none`: Standard passthrough (live calls, no recording)

### Header Sanitization
**Decision**: Custom middleware before VCR strips headers
- Strip: Authorization, Cookie, x-api-key
- Also strip any header matching *-token, *-secret patterns
- Log sanitization for audit

### Cassette Storage
**Decision**: Store in tests/cassettes/ directory
- File naming: {test_name}.yaml
- Git-crypt encryption enabled via .gitattributes
- CI unlocks before test run

## Risks / Trade-offs

- **Risk**: Cassettes become stale when API changes
  - **Mitigation**: Document refresh process; run record mode periodically

- **Risk**: Large cassette files bloat repo
  - **Mitigation**: Git-crypt reduces readability; consider LFS if needed

- **Risk**: git-crypt not available on all systems
  - **Mitigation**: Document installation requirement; CI handles it