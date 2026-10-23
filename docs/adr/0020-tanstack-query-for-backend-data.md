# ADR 0020: TanStack Query for Backend Data in the Frontend

- Status: Accepted
- Date: 2026-10-23

## Context

The frontend loaded backend data through six hand-written patterns: a TTL cache with a disk fallback, a generic resource hook, a module-level promise with listeners, two hooks with their own request counters and a week loader with its own cache, retry timer and edit counter.
Each pattern guarded against superseded responses in its own way, and each new freshness requirement, such as the periodic refresh of ADR 0018, grew one of them by hundreds of lines.

### Evaluated Options
- TanStack Query
  - Pros: Caching, request deduplication, invalidation, prefetching, optimistic updates, retries and refetch intervals are configuration; widely used and documented.
  - Cons: A new dependency of about 13 kB gzipped, and its own concepts to learn.
- SWR
  - Pros: Smaller and simpler.
  - Cons: No mutation lifecycle for optimistic updates with rollback, which the week grid needs.
- Keep the hand-written caches
  - Pros: No dependency.
  - Cons: Every pattern keeps its own race handling and tests, and freshness features stay expensive.

## Decision

- Components read backend data through query hooks in `src/app/hooks` that call the services in `src/services`.
- One `QueryClient` holds all frontend caches: data stays fresh until a write or "Daten neu laden" invalidates it, without retries or refetch on focus unless a query opts in.
- The Rust backend remains the only persistent cache; the frontend keeps no cache of its own outside the query client.

## Consequences

- The TTL cache, the generic resource hook and the hand-written loading hooks are removed as their data moves to queries.
- Components that only render receive data as props, so their tests need no query client.
