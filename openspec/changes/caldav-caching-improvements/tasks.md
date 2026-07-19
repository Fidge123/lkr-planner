## 1. Backend cache module

- [ ] 1.1 Write failing `cargo test`s for `CaldavEventCache`: cache miss, cache hit within TTL, cache miss after TTL expiry, targeted invalidation
- [ ] 1.2 Implement `CaldavEventCache` in `src-tauri/src/integrations/calendar/` keyed by `(employee_id, calendar_url, week_start)` with 30s TTL, satisfying the tests
- [ ] 1.3 Write failing tests for request coalescing of concurrent identical fetches
- [ ] 1.4 Implement in-flight request coalescing for the cache, satisfying the tests
- [ ] 1.5 Write failing tests for stale-on-error fallback (return last good entry when a refresh fails)
- [ ] 1.6 Implement stale-on-error fallback, satisfying the tests

## 2. Shared HTTP client

- [ ] 2.1 Add a shared `reqwest::Client` held in Tauri managed state (or `OnceLock`), following the existing Daylite integration's pattern
- [ ] 2.2 Replace per-call `reqwest::Client::new()` in `load_week_events`, `create_assignment`, `update_assignment`, and `delete_assignment` (`src-tauri/src/integrations/calendar/commands.rs`) with the shared client

## 3. Wire cache into commands

- [ ] 3.1 Update `load_week_events` to consult `CaldavEventCache` before issuing CalDAV requests, storing fresh results on miss
- [ ] 3.2 Add targeted cache invalidation calls to `create_assignment`, `update_assignment`, and `delete_assignment` for the affected employee/week
- [ ] 3.3 Add `cargo test` coverage for week-boundary edge cases when deriving the invalidation key from an event's date

## 4. Concurrent Daylite project resolution

- [ ] 4.1 Change the sequential loop resolving missing Daylite project refs in `load_week_events` (`commands.rs`) to resolve them concurrently

## 5. Frontend targeted invalidation

- [ ] 5.1 Replace the full-cache wipe (`reloadAssignments()`) after writes in `src/app/hooks/use-planning-assignments.ts` with a targeted invalidation of the affected week
- [ ] 5.2 Add/update TS unit tests (`bun test`) covering targeted invalidation leaves other cached weeks untouched

## 6. Documentation

- [ ] 6.1 Write ADR `docs/adr/0013-caldav-event-caching.md` documenting the backend cache decision, following the ADR 0007 format
- [ ] 6.2 Run `bun run test:docs` after adding the ADR

## 7. Verification

- [ ] 7.1 Run `cargo test` and confirm all new and existing CalDAV tests pass
- [ ] 7.2 Run `bun test` and confirm all new and existing frontend tests pass
- [ ] 7.3 Run `bun lint` and fix any issues
- [ ] 7.4 Manually verify in the running app: week navigation is faster on repeat visits, and an edit is reflected immediately without a full reload
