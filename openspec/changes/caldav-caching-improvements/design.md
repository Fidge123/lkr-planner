## Context

`load_week_events` (`src-tauri/src/integrations/calendar/commands.rs`) fetches every employee's primary and absence calendar directly from CalDAV on each call, with no backend cache.
A new `reqwest::Client` is constructed per command call (`load_week_events`, `create_assignment`, `update_assignment`, `delete_assignment`), discarding connection keep-alive.
The frontend hook `use-planning-assignments.ts` already keeps an in-memory, per-week cache with no TTL, prefetches adjacent weeks, and wipes the entire cache via `reloadAssignments()` after any write.
ADR 0007 already established the pattern for this kind of cache (TTL-based, in-memory, request coalescing, stale-on-error fallback) for Daylite project lookups; this change applies the same pattern to CalDAV week events, but in the Rust backend rather than the frontend, since CalDAV is called exclusively from Tauri commands.

## Goals / Non-Goals

**Goals:**
- Avoid redundant CalDAV `REPORT` requests for week ranges already fetched recently.
- Reuse one `reqwest::Client` across all CalDAV commands for connection reuse.
- Invalidate only the affected employee/week cache entry on write, instead of a full wipe.
- Coalesce concurrent identical week fetches (e.g. rapid navigation) into a single in-flight request.

**Non-Goals:**
- No durable/offline cache; the cache remains process-local in-memory, matching ADR 0007's scope for Daylite.
- No change to the CalDAV wire protocol, VEVENT format, or write semantics (create/update/delete still hit CalDAV directly, then invalidate the cache).
- No change to the frontend's own week-cache data structure beyond how it invalidates entries after writes.

## Decisions

### Backend cache location and shape
Add a `CaldavEventCache` in `src-tauri/src/integrations/calendar/`, keyed by `(employee_id, calendar_url, week_start)`, storing the resolved event list and a `fetched_at` timestamp.
Default TTL: 30 seconds, matching ADR 0007's Daylite cache, since CalDAV data changes at a similar human-editing cadence.
Alternative considered: caching at the raw CalDAV response (XML) level instead of resolved events. Rejected because resolved events already include Daylite project references, which are the expensive part to recompute, and caching post-resolution avoids re-running that resolution on every cache hit.

### Shared HTTP client
Move `reqwest::Client` construction into a single `once_cell`/`OnceLock`-backed singleton (or pass a shared client held in Tauri managed state), replacing the per-call `reqwest::Client::new()` in `commands.rs`.
Alternative considered: leaving per-call clients but tuning connection pool settings. Rejected because it does not remove the per-call TCP/TLS handshake overhead that a shared client's connection pool avoids.

### Invalidation strategy
On `create_assignment` / `update_assignment` / `delete_assignment`, invalidate only the cache entry for `(employee_id, calendar_url, week_start)` derived from the affected event's date, then let the next read repopulate it.
The frontend's `reloadAssignments()` full-wipe call is replaced with a targeted call that clears only the affected week from its own cache, mirroring the backend's targeted invalidation so both layers stay consistent.
Alternative considered: TTL-only invalidation (no explicit invalidation on write). Rejected because a 30s stale window after a user's own edit would be a visible regression versus current behavior, where writes are followed by an immediate full reload.

### Request coalescing
Use an in-flight request map (`(employee_id, calendar_url, week_start) -> shared future`) so concurrent identical fetches await the same CalDAV request rather than issuing duplicates, following the same coalescing approach as ADR 0007.

## Risks / Trade-offs

- [Stale data within TTL window if CalDAV is edited outside the app] → Mitigate by keeping TTL short (30s) and the existing manual week-navigation refetch behavior; users editing CalDAV externally already tolerate light staleness under the frontend cache today.
- [Shared `reqwest::Client` state adds Tauri-managed-state complexity] → Mitigate by following the existing Daylite integration's pattern for holding shared clients/caches in managed state.
- [Targeted invalidation depends on correctly deriving the week key from the affected event's date] → Mitigate with unit tests covering week-boundary edge cases (event near week start/end).

## Migration Plan

- Implement the backend cache and shared client behind the existing `load_week_events`/`create_assignment`/`update_assignment`/`delete_assignment` command signatures — no frontend contract changes required beyond invalidation calls.
- Add `cargo test` coverage for cache hit/miss/TTL expiry/invalidation before wiring it into the commands (red/green TDD per CLAUDE.md).
- Update `use-planning-assignments.ts` to call targeted invalidation instead of `reloadAssignments()` after writes.
- Document the decision as a new ADR once implemented, following the numbering after `0012`.
- No rollback data migration needed since the cache is in-memory only; reverting the commit fully reverts behavior.

## Open Questions

None — confirmed with the user that a fixed 30s TTL constant is sufficient for this change; no configurable TTL is needed.
