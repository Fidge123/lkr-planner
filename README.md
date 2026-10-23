# LKR Planner

LKR Planner is a macOS desktop application for weekly project planning with Daylite as the system of record.
It is built with Tauri, React, TypeScript, and Bun.

## Scope

The app plans assignments in a weekly grid, stores them in the employees' ZEP CalDAV calendars and mirrors planned project days as Daylite appointments.
Implemented behavior is specified in [openspec/specs](openspec/specs), planned work lives in [openspec/changes](openspec/changes), and architecture decisions are recorded in [docs/adr](docs/adr).

## Product Direction

Daylite remains the source of truth for projects and employees.
The app supports a planner workflow where assignments are managed in a weekly view, enriched with ZEP CalDAV context, and synchronized to external systems where needed.

### Daylite

The application integrates with the [Daylite API](https://developer.daylite.app/reference/getting-started).
Daylite is the source of truth for project and employee master data.
The refresh-token rotation is documented in [docs/daylite-authentication-flow.md](docs/daylite-authentication-flow.md).
After every assignment create, update, move and delete, a background task links the edited worker to the project's `Einsatz <date>` appointment for that day and unlinks the worker from projects it left, as recorded in [ADR 0017](docs/adr/0017-daylite-appointments-mirror-assignments.md).

### Planradar

The application is planned to integrate with the [Planradar API](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs).
Planradar synchronization is not implemented yet.
The active backlog covers the client foundation plus flows to compare, link existing projects, create new linked projects, and reactivate archived linked projects.

### iCal / CalDAV

ZEP CalDAV calendars are used as planning context and as the synchronization target for employee assignments.
Current implementation covers storing employee primary and absence calendar URLs in Daylite, validating and diagnosing those connections, rendering absences and assignments in the weekly grid, and writing assignment create/update/delete operations back to CalDAV.

## Development

### Local Quality Workflow

CI runs these checks, so run them before pushing:

```bash
# Frontend and script tests, once and in six timezones
bun test
bun run test:timezones

# Biome lint and format, applying safe fixes
bun lint

# Rust format, lint and tests
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml

# OpenSpec specs and changes
bunx openspec validate --all --strict
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

The live-record harness is an ignored Rust test in [recording_harness.rs](src-tauri/src/integrations/daylite/recording_harness.rs).
It writes the standard Daylite cassette files in `tests/cassettes/` using the real API.

Required environment variables:

- `VCR_MODE=record`
- `DAYLITE_BASE_URL`
- `DAYLITE_REFRESH_TOKEN`
- `DAYLITE_VCR_PROJECT_SEARCH_TERM` (a term expected to match at least one project)
- `DAYLITE_VCR_PROJECT_NO_MATCH_SEARCH_TERM` (a term expected to match no projects, to capture Daylite's empty-result response shape)

Optional environment variables:

- `DAYLITE_VCR_SCOPE=readonly|all`

`DAYLITE_VCR_SCOPE` defaults to `readonly`.
That records only the read-only cassettes:

- `daylite-refresh-tokens.json`
- `daylite-search-projects.json`
- `daylite-overdue-projects.json`
- `daylite-list-contacts.json`

Use `DAYLITE_VCR_SCOPE=all` only when you intentionally want to refresh the mutating cassettes `daylite-update-contact-ical-urls.json` and `daylite-appointments.json`.
That mode additionally requires:

- `DAYLITE_VCR_CONTACT_REFERENCE` (a test contact whose calendar URLs are rewritten and that is linked to the recorded appointment)
- `DAYLITE_VCR_PRIMARY_ICAL_URL` (the primary calendar URL written to that contact)
- `DAYLITE_VCR_ABSENCE_ICAL_URL` (the absence calendar URL written to that contact)
- `DAYLITE_VCR_APPOINTMENT_PROJECT_REFERENCE` (a test project the recorded appointment is linked to)

The contact recording writes the given calendar URLs to the contact, so pass its real URLs if the contact appears in the planning grid.
The appointment recording creates an app-managed appointment on 2030-01-07 linked to that test project and the test contact, searches and edits it, unlinks the contact and deletes it, which leaves it in the Daylite trash.

Read-only recording example:

```bash
VCR_MODE=record \
DAYLITE_VCR_SCOPE=readonly \
DAYLITE_BASE_URL="https://api.marketcircle.net/v1" \
DAYLITE_REFRESH_TOKEN="..." \
DAYLITE_VCR_PROJECT_SEARCH_TERM="Nord" \
DAYLITE_VCR_PROJECT_NO_MATCH_SEARCH_TERM="XXXXX" \
cargo test --manifest-path src-tauri/Cargo.toml \
  record_daylite_cassettes_from_live_api -- --ignored --nocapture
```

Record all cassettes, including the live contact PATCH and appointment cassettes:

```bash
VCR_MODE=record \
DAYLITE_VCR_SCOPE=all \
DAYLITE_BASE_URL="https://api.marketcircle.net/v1" \
DAYLITE_REFRESH_TOKEN="..." \
DAYLITE_VCR_PROJECT_SEARCH_TERM="Nord" \
DAYLITE_VCR_PROJECT_NO_MATCH_SEARCH_TERM="XXXXX" \
DAYLITE_VCR_CONTACT_REFERENCE="/v1/contacts/500" \
DAYLITE_VCR_PRIMARY_ICAL_URL="https://example.com/primary.ics" \
DAYLITE_VCR_ABSENCE_ICAL_URL="https://example.com/absence.ics" \
DAYLITE_VCR_APPOINTMENT_PROJECT_REFERENCE="/v1/projects/900" \
cargo test --manifest-path src-tauri/Cargo.toml \
  record_daylite_cassettes_from_live_api -- --ignored --nocapture
```

After recording:

1. Inspect the updated cassette JSON and confirm no `Authorization`, `Cookie`, or `x-api-key` values were written.
2. Replay the full Rust suite without `VCR_MODE`; a replay test fails when its cassette is missing, so commit every recorded cassette:

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
