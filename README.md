# LKR Planner

LKR Planner is a macOS desktop application for weekly project planning with Daylite as the system of record.
It is built with Tauri, React, TypeScript, and Bun.

## Current Scope

The current implementation focuses on the technical and domain foundation for planning.
The backlog is currently centered on weekly assignment workflows, iCal synchronization, and Planradar project linking.

### Implemented

- Daylite authentication with refresh-token rotation and documented token flow in [docs/daylite-authentication-flow.md](/Users/flori/dev/lkr-planner/docs/daylite-authentication-flow.md)
- Typed Daylite API foundation for project and contact read/search operations
- Local application store for persisted configuration and cached integration data
- Employee source switched to Daylite contacts (`Monteur` category)
- Employee iCal URLs are read from and written back to Daylite contact `urls`
- Daylite project overview rendered below the planning table
- ADR-based architecture documentation in [docs/adr](/Users/flori/dev/lkr-planner/docs/adr)

### Planned / In Backlog

- Validate and test employee primary and absence iCal sources
- Replace remaining dummy assignment data with persisted planning state
- Assignment modal CRUD flow for weekly planning
- Default suggestions and live Daylite project search in the assignment modal
- Next-day quick-add suggestion behavior
- German holiday import for week view
- Calendar cell composition and rendering for assignments, holidays, absences, and appointments
- Deterministic daily time-slot allocation for assignment sync
- iCal synchronization for assignment create/update/delete operations
- Planradar API client, existing-project linking, project creation, and archived-project reactivation
- Secure OS-level token storage instead of plain-text local persistence
- Record/replay HTTP testing infrastructure for integration tests

## Product Direction

Daylite remains the source of truth for projects and employees.
The app is intended to support a planner workflow where assignments are managed in a weekly view, enriched with iCal context, and later synchronized to external systems where needed.

### Daylite

The application integrates with the [Daylite API](https://developer.daylite.app/reference/getting-started).
Daylite is the source of truth for project and employee master data.
Current work already covers authentication, token refresh handling, project reads, and employee contact/iCal configuration.

### Planradar

The application is planned to integrate with the [Planradar API](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs).
Planradar synchronization is not implemented yet.
The active backlog covers the client foundation plus flows to link existing projects, create new linked projects, and reactivate archived linked projects.

### iCal

iCal is used as planning context and as a future synchronization target for employee assignments.
Current implementation covers storing the two employee iCal URLs in Daylite.
Validation, diagnostics, cell composition, and write synchronization are still planned backlog items.

## Development

### Local Quality Workflow

Before committing changes, ensure all quality checks pass:

```bash
# Run tests
bun test

# Run tests in watch mode during development
bun test:watch

# Check code quality (lint)
bun lint

# Auto-fix linting issues
bun lint:fix

# Check code formatting
bun format:check

# Auto-format code
bun format
```

### HTTP Cassette Tests

Rust HTTP cassette tests use JSON fixtures in `tests/cassettes/`.
Replay mode is the default.
Record mode is enabled with `VCR_MODE=record`.

#### Dependencies

- `git-crypt` must be installed locally to unlock encrypted cassette files.
- CI also requires `git-crypt` plus a repository secret named `GIT_CRYPT_KEY_B64`.

```bash
# macOS
brew install git-crypt

git-crypt init
```

If your checkout is locked, unlock cassette files before running Rust tests:

```bash
git-crypt unlock /path/to/git-crypt-key
```

#### Run Replay Mode

```bash
# Full Rust suite (default replay mode)
cargo test --manifest-path src-tauri/Cargo.toml

# Replay-only cassette test
cargo test --manifest-path src-tauri/Cargo.toml \
  integrations::daylite::client::tests::replays_recorded_response_without_network_call
```

#### Run Record Mode

```bash
VCR_MODE=record cargo test --manifest-path src-tauri/Cargo.toml \
  integrations::daylite::client::tests::records_sanitized_cassette_in_record_mode
```

When adding or refreshing committed cassette files, keep them under `tests/cassettes/` so `.gitattributes` applies `git-crypt` automatically.

#### Create the CI Secret

Create a symmetric git-crypt key, base64-encode it, and store the encoded value as the GitHub Actions repository secret `GIT_CRYPT_KEY_B64`:

```bash
git-crypt export-key /tmp/lkr-planner-git-crypt.key
base64 < /tmp/lkr-planner-git-crypt.key | tr -d '\n'
rm /tmp/lkr-planner-git-crypt.key
```

On macOS, instead of printing the encoded value to stdout, you can copy it directly into the clipboard before deleting the key file:

```bash
base64 < /tmp/lkr-planner-git-crypt.key | tr -d '\n' | pbcopy
```

### Running the Application

```bash
# Development mode
bun tauri dev

# Build for macOS
bun build:macos
```

**Note:** The same quality checks (`bun test`, `bun lint`, `bun format:check`) are run in CI/CD, so running them locally ensures your changes will pass automated checks.
