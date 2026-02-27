# ADR 0009: Rust Planning Command Contracts for Daylite Data

- Status: Accepted
- Date: 2026-02-27

## Context

Frontend services currently compensate for raw Daylite payloads with TypeScript mapping, normalization, filtering, and local-store cache persistence logic.
This duplicates integration logic across frontend and backend boundaries and couples frontend services to raw command payload details.
The migration goal is to keep frontend services focused on orchestration concerns (TTL cache, stale fallback, retries) while Rust commands return planning-ready records.

### Evaluated Options
- Keep normalization and cache persistence in frontend services
  - Pros: No Rust contract break and minimal backend changes.
  - Cons: Continues duplicated mapping logic, wider frontend coupling to raw API/store payloads, and harder long-term consistency.
- Add new v2 commands while keeping old contracts
  - Pros: Safer incremental migration path with compatibility window.
  - Cons: Temporary command duplication and additional maintenance overhead until old commands are removed.
- Replace existing command contracts with planning-ready Rust DTOs
  - Pros: Single source of truth for Daylite normalization/filtering and cleaner frontend services.
  - Cons: Breaking command payload changes requiring coordinated frontend and generated type updates.

## Decision

Replace existing `daylite_list_projects`, `daylite_list_contacts`, and `daylite_update_contact_ical_urls` command outputs with planning-ready DTOs provided by Rust.
Add `daylite_list_cached_contacts` in Rust so frontend services no longer read/write raw `LocalStore` for contact cache usage.
Keep `Monteur` filtering hardcoded in Rust backend contact flows.
Keep search commands unchanged unless required by compile-time coupling.

## Consequences

- Frontend services consume generated planning-ready command types and remove redundant payload normalization/filtering logic.
- Contact cache persistence ownership moves to Rust command handlers.
- Generated TypeScript bindings change and require coordinated frontend test updates.
- Command contract changes are breaking for any clients expecting previous raw payloads.
