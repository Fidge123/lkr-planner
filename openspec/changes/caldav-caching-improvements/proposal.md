## Why

Every week navigation re-issues full CalDAV `REPORT` requests for each employee's primary and absence calendar, and every Tauri command opens a brand-new `reqwest::Client`.
There is no backend cache at all, so responsiveness depends entirely on the frontend's in-memory, per-session week cache, which is wiped completely on any write.
Introducing a backend-side cache with targeted invalidation removes redundant network round-trips and keeps the UI responsive after edits.

## What Changes

- Add an in-memory, TTL-based cache in the Rust backend for CalDAV week event fetches, keyed by employee + calendar URL + week range.
- Reuse a single shared `reqwest::Client` instance across CalDAV commands instead of constructing one per call.
- Coalesce concurrent identical fetch requests (e.g. rapid week navigation) into a single in-flight CalDAV request.
- Replace the frontend's full-cache wipe on write (`reloadAssignments()`) with targeted invalidation of only the affected employee/week entry, backed by the new backend cache.
- Fetch missing Daylite project references concurrently instead of sequentially when resolving events for a loaded week.

## Capabilities

### New Capabilities
- `caldav-event-caching`: Backend-side caching layer for CalDAV week event fetches, covering TTL-based freshness, targeted invalidation on write, request coalescing, and shared HTTP client reuse.

### Modified Capabilities
- `assignment-persistence`: "Load assignments from CalDAV" and "Week navigation with live data" requirements change from always-live CalDAV queries with frontend-only prefetching to cache-backed loads that also serve fresh data after targeted invalidation.

## Impact

- `src-tauri/src/integrations/calendar/commands.rs`: add cache lookup/store around `load_week_events`, shared `reqwest::Client`, targeted invalidation on `create_assignment`/`update_assignment`/`delete_assignment`.
- `src-tauri/src/integrations/calendar/caldav.rs`: no protocol changes expected; caching sits above the HTTP layer.
- `src/app/hooks/use-planning-assignments.ts`: replace full-cache wipe with targeted invalidation calls after write operations.
- `docs/adr/`: new ADR documenting the backend caching strategy (TTL, invalidation, client reuse), following the precedent set by ADR 0007 (Daylite project cache).
- Tests: new `cargo test` coverage for cache hit/miss/invalidation behavior; existing CalDAV VCR-style tests remain unaffected since caching sits above the HTTP transport.
