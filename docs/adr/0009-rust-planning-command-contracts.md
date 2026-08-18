# ADR 0009: Rust Planning Command Contracts for Daylite Data

- Status: Accepted
- Date: 2026-02-27
- Amended: 2026-08-14 (two named commands removed, category color joined at render time)

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

## Amendment 2026-08-14

`daylite_list_projects` and `daylite_update_contact_ical_urls`, two of the three commands named above, have been removed.
Neither had a frontend caller: project resolution goes through the single-project read, and the iCal write reaches Daylite through the ZEP calendar flow.
`daylite_list_contacts` and `daylite_list_cached_contacts` are unaffected, as is the decision that Rust returns planning-ready records.

The category color is now joined where the card is rendered rather than in the week payload.
`CalendarCellEvent` carries `projectCategory`, and the planning grid looks the color up in the same map the assignment modal's project picker already loads.
Fetching and normalizing the categories stays in Rust, so only the join moved, not the integration logic.

- Every week load previously fetched the whole category map again, three times per week navigation once prefetching is counted, for data that changes about as often as configuration.
- The frontend fetched the same map separately for the project picker, so the two paths duplicated each other.
- The color map is read once per session and reset by "Daten neu laden", so a recolored category appears after a reload or a restart rather than on the next week load.
  This is accepted deliberately: category colors are closer to configuration than to planning data.
- A project's own category still comes from the project resolution cache, so recategorizing a project surfaces as quickly as any other project change.

## Consequences

- Frontend services consume generated planning-ready command types and remove redundant payload normalization/filtering logic.
- Contact cache persistence ownership moves to Rust command handlers.
- Generated TypeScript bindings change and require coordinated frontend test updates.
- Command contract changes are breaking for any clients expecting previous raw payloads.
