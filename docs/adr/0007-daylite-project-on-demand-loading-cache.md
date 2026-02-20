# ADR 0007: Daylite Project On-Demand Loading with 30s In-Memory Cache

- Status: Accepted
- Date: 2026-02-15

## Context
BL-007 requires replacing dummy project data in planning with live Daylite reads.
The app should avoid repeated identical project requests in short intervals, but Daylite remains the Source of Truth and no offline-first behavior is required.

## Decision
- Load Daylite projects on demand when planning needs project data.
- Use a short-lived in-memory cache with a default TTL of 30 seconds.
- Coalesce concurrent identical reads into one in-flight backend request.
- If a refresh fails and there is a previous successful cache entry, keep rendering cached data and expose a German error state with retry.

## Consequences
- Repeated UI reads inside 30 seconds do not create redundant Daylite calls.
- Planning remains usable with the last known project list during transient Daylite errors.
- Cache is process-local and intentionally not a durable offline store.
- Future work can wire TTL configuration to persisted local settings without changing the loading contract.
