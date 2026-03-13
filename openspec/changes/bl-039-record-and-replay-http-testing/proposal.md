## Why

Testing external API integrations (Planradar, Daylite) requires deterministic, fast tests without live network calls. A VCR-style infrastructure records HTTP interactions and replays them for subsequent test runs.

## What Changes

- Integrate VCR middleware into Rust reqwest client
- Add VCR_MODE environment variable toggle (record/replay)
- Implement header sanitization to strip auth before recording
- Set up git-crypt for encrypted cassette storage

## Capabilities

### New Capabilities
- `http-recording`: Record HTTP interactions to cassette files
- `http-replay`: Replay recorded HTTP interactions
- `header-sanitization`: Strip sensitive headers before recording

### Modified Capabilities
- `planradar-client`: Extended to support VCR mode
- `daylite-client`: Extended to support VCR mode

## Impact

- Code: New Rust middleware for VCR, header scrubbing hook
- Dependencies: reqwest-middleware, reqwest-vcr (dev), git-crypt
- CI: Update CI to unlock git-crypt before tests