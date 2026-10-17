## 1. Shared Daylite client

- [x] 1.1 Write a failing `cargo test` in `daylite/client.rs`: two `DayliteApiClient`s sending one request each to a local keep-alive server open a single connection
- [x] 1.2 Back `ReqwestTransport` with one process-wide `reqwest::Client`, satisfying the test

## 2. Persisted stale-while-revalidate project cache

- [x] 2.1 Write failing tests in `daylite/project_cache.rs`: `peek` returns an entry of any age, a stale entry is returned when its refresh fails, restored entries are stale, a forced refresh bypasses a fresh entry, a snapshot lists every known project
- [x] 2.2 Rework `ProjectCache` with a five minute freshness window, separate known value and load lock per slot, restore and snapshot, satisfying the tests
- [x] 2.3 Write failing tests for reading and writing the cache file, including an unreadable file being ignored
- [x] 2.4 Persist the cache to `daylite-projects.json` in the app cache directory after a resolution that fetched, and restore it at startup

## 3. Two-phase week loading

- [x] 3.1 Write failing tests in `calendar/events/resolve.rs`: a cached project resolves the assignment, an uncached one keeps the summary and is pending, bare events are never pending
- [x] 3.2 Add `project_pending` to `CalendarCellEvent` and resolve `load_week_events` from the cache only, satisfying the tests
- [x] 3.3 Write failing tests for limiting a week load to a set of employees
- [x] 3.4 Add the employee filter and the load purpose to `load_week_events`
- [x] 3.5 Add the `resolve_week_projects` command with telemetry and `force_refresh`
- [x] 3.6 Write failing `bun test`s for patching resolved projects into a week and clearing the pending flags
- [x] 3.7 Call `resolve_week_projects` after every displayed, cached, and prefetched week and patch the results
- [x] 3.8 Show pending assignments without the unresolved warning and keep them undraggable until resolved

## 4. Startup sequence

- [x] 4.1 Write failing `bun test`s for detecting a changed calendar configuration between two settings reads
- [x] 4.2 Read the settings before the first week load and reload after the contact sync only on a change

## 5. Optimistic edits with targeted refresh

- [x] 5.1 Write failing `bun test`s for removing and relocating an event in a week, including renumbering the affected cells
- [x] 5.2 Write failing tests for merging refreshed employee rows and discarding a refresh that an optimistic edit superseded
- [x] 5.3 Relocate a dropped card before the write and refresh the source and target rows afterwards
- [x] 5.4 Remove a deleted card and refresh only the edited employee's row after modal and quick-add writes

## 6. Telemetry

- [x] 6.1 Write failing tests for the new operation names
- [x] 6.2 Record prefetch and refresh loads and project resolution under their own operations

## 7. ZEP credentials and CalDAV client

- [x] 7.1 Write failing tests for the in-memory credential cache: a second read skips the keychain, a save replaces the cached value
- [x] 7.2 Serve ZEP credentials from memory and send discovery and probes through the shared CalDAV client

## 8. Documentation

- [x] 8.1 Write ADR 0015 for two-phase loading with the persisted project cache, superseding ADR 0007 and amending ADR 0011
- [x] 8.2 Write ADR 0016 for optimistic edits with targeted refresh
- [x] 8.3 Run `bun run test:docs`

## 9. Verification

- [x] 9.1 Run `cargo test` and `cargo clippy --all-targets -- -D warnings`
- [x] 9.2 Run `bun test`, `bunx tsc --noEmit`, and `bunx biome ci .`
- [x] 9.3 Run `bunx openspec validate --all --strict`
