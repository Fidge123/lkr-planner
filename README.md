# LKR Planner

LKR Planner is a macOS desktop application for weekly project planning with Daylite as the system of record.
It is built with Tauri, React, TypeScript, and Bun.

## Current Scope

The current implementation covers the weekly planning view with full assignment CRUD, CalDAV synchronization, Daylite project/employee data, and German holiday display.
The backlog is now centered on Planradar project linking, assignment modal quality-of-life features, and drag-and-drop replanning.

### Implemented

- Daylite authentication with refresh-token rotation and documented token flow in [docs/daylite-authentication-flow.md](/Users/flori/dev/lkr-planner/docs/daylite-authentication-flow.md)
- Typed Daylite API foundation for project and contact read/search operations
- Local application store for persisted configuration and cached integration data
- Secure OS-level token and credential storage via the system keychain (`keyring-core`)
- Employee source switched to Daylite contacts (`Monteur` category)
- Employee ZEP CalDAV calendar URLs are read from and written back to Daylite contact `urls`, with connection validation and diagnostics
- Daylite project overview rendered below the planning table
- Weekly planning table backed by real, persisted assignment data instead of dummy data
- Assignment modal CRUD flow (create, edit, delete) for weekly planning
- Live Daylite project search/filter in the assignment modal
- Calendar cell composition and rendering for assignments, holidays, absences, and appointments
- Employee absence display from the ZEP absence calendar
- German holiday import for the week view (Nager.Date API)
- Weekend visibility toggle for the planning table
- CalDAV synchronization for assignment create/update/delete operations
- Record/replay HTTP (VCR) testing infrastructure for integration tests
- ADR-based architecture documentation in [docs/adr](/Users/flori/dev/lkr-planner/docs/adr)

### Planned / In Backlog

- Default suggestions in the assignment modal
- Next-day quick-add suggestion behavior
- Deterministic daily time-slot allocation for assignment sync
- Drag-and-drop of assignment cards across days and employees, including in-day reordering
- Planradar API client, existing-project linking, project creation, and archived-project reactivation
- Daylite-to-Planradar project comparison

## Product Direction

Daylite remains the source of truth for projects and employees.
The app supports a planner workflow where assignments are managed in a weekly view, enriched with ZEP CalDAV context, and synchronized to external systems where needed.

### Daylite

The application integrates with the [Daylite API](https://developer.daylite.app/reference/getting-started).
Daylite is the source of truth for project and employee master data.
Current work covers authentication, token refresh handling, project reads, and employee contact/iCal configuration.

### Planradar

The application is planned to integrate with the [Planradar API](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs).
Planradar synchronization is not implemented yet.
The active backlog covers the client foundation plus flows to compare, link existing projects, create new linked projects, and reactivate archived linked projects.

### iCal / CalDAV

ZEP CalDAV calendars are used as planning context and as the synchronization target for employee assignments.
Current implementation covers storing employee primary and absence calendar URLs in Daylite, validating and diagnosing those connections, rendering absences and assignments in the weekly grid, and writing assignment create/update/delete operations back to CalDAV.

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
```

The repository is already configured for `git-crypt`.
Most contributors only need to install it and unlock the checkout.

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

#### Record Live Daylite Cassettes

The live-record harness is an ignored Rust test in [recording_harness.rs](/Users/flori/dev/lkr-planner/src-tauri/src/integrations/daylite/recording_harness.rs#L74).
It writes the standard Daylite cassette files in `tests/cassettes/` using the real API.

Required environment variables:

- `VCR_MODE=record`
- `DAYLITE_BASE_URL`
- `DAYLITE_REFRESH_TOKEN`
- `DAYLITE_VCR_PROJECT_SEARCH_TERM` (a term expected to match at least one project)
- `DAYLITE_VCR_PROJECT_NO_MATCH_SEARCH_TERM` (a term expected to match no projects, to capture Daylite's empty-result response shape)

Optional environment variables:

- `DAYLITE_VCR_SCOPE=readonly|all`
- `DAYLITE_VCR_CONTACT_REFERENCE`
- `DAYLITE_VCR_PRIMARY_ICAL_URL`
- `DAYLITE_VCR_ABSENCE_ICAL_URL`

`DAYLITE_VCR_SCOPE` defaults to `readonly`.
That records only the read-only cassettes:

- `daylite-refresh-tokens.json`
- `daylite-list-projects.json`
- `daylite-search-projects.json`
- `daylite-list-contacts.json`

Use `DAYLITE_VCR_SCOPE=all` only when you intentionally want to refresh the mutating cassette `daylite-update-contact-ical-urls.json`.
For that mode you must also provide `DAYLITE_VCR_CONTACT_REFERENCE`, `DAYLITE_VCR_PRIMARY_ICAL_URL`, and `DAYLITE_VCR_ABSENCE_ICAL_URL`.

Read-only recording example:

```bash
VCR_MODE=record \
DAYLITE_VCR_SCOPE=readonly \
DAYLITE_BASE_URL="https://app.daylite.app/api/v1" \
DAYLITE_REFRESH_TOKEN="..." \
DAYLITE_VCR_PROJECT_SEARCH_TERM="Nord" \
DAYLITE_VCR_PROJECT_NO_MATCH_SEARCH_TERM="XXXXX" \
cargo test --manifest-path src-tauri/Cargo.toml \
  record_daylite_cassettes_from_live_api -- --ignored --nocapture
```

Record all cassettes, including the live contact PATCH cassette:

```bash
VCR_MODE=record \
DAYLITE_VCR_SCOPE=all \
DAYLITE_BASE_URL="https://app.daylite.app/api/v1" \
DAYLITE_REFRESH_TOKEN="..." \
DAYLITE_VCR_PROJECT_SEARCH_TERM="Nord" \
DAYLITE_VCR_PROJECT_NO_MATCH_SEARCH_TERM="XXXXX" \
DAYLITE_VCR_CONTACT_REFERENCE="/v1/contacts/500" \
DAYLITE_VCR_PRIMARY_ICAL_URL="https://example.com/primary.ics" \
DAYLITE_VCR_ABSENCE_ICAL_URL="https://example.com/absence.ics" \
cargo test --manifest-path src-tauri/Cargo.toml \
  record_daylite_cassettes_from_live_api -- --ignored --nocapture
```

After recording:

1. Inspect the updated cassette JSON and confirm no `Authorization`, `Cookie`, or `x-api-key` values were written.
2. Replay the full Rust suite without `VCR_MODE`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

3. Confirm Git still treats the fixtures as encrypted:

```bash
git-crypt status -e tests/cassettes
```

### CalDAV Integration Test (disposable Radicale server)

The CalDAV assignment write path (create, update, delete) is covered end-to-end by `caldav_write_path_against_disposable_radicale`.
It runs over real HTTP against a throwaway [Radicale](https://radicale.org/) server, with no production credentials.
The test spawns Radicale on a random port, seeds a calendar, discovers it by display name, runs create -> update -> delete, and tears everything down.

Calendar discovery (a PROPFIND that finds a collection by its display name) exists only inside this test, so it can find the calendar it just seeded on a throwaway server.
It is test-only infrastructure and does not change how the running app resolves a calendar URL, which is still read from configuration.

This test is mandatory, not skippable: it uses [`uv`](https://docs.astral.sh/uv/) to fetch and run Radicale on demand via `uvx radicale`, so no manual install step is needed, but it fails loudly if `uv`/`uvx` is missing, the on-demand install fails, or the server never becomes ready.

Install `uv` once to run it locally:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

CI installs `uv` via `astral-sh/setup-uv` and runs the test automatically; `uvx` caches Radicale after the first run, so later runs start instantly.

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
