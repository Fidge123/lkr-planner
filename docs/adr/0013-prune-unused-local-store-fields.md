# ADR 0013: Prune Unused Local Store Fields

- Status: Accepted
- Date: 2026-06-05

## Context

The local store schema defined in ADR 0005 (`local-store.json`) accumulated
fields that are no longer read or written by any business logic. Three forces
converged:

- **Tokens moved to the OS keychain.** ADR 0006 originally persisted Daylite
  tokens in `tokenReferences.*`. Token state is now stored via
  `secret_manager` (keychain), read through `load_daylite_tokens` and written
  through `store_daylite_tokens`. The `tokenReferences` struct was left behind
  and only ever serialized as empty strings. ZEP credentials are likewise kept
  in the keychain; only the non-secret `zepCaldavRootUrl` stays in the store.
- **Contact filtering became fixed.** ADR 0012 replaced the configurable
  `contactFilter.activeEmployeeKeyword` ("Monteur") with a fixed Daylite
  search over both planning categories ("Monteur" and "Test") in
  `list_contacts_core`. The keyword field is no longer consulted.
- **Unimplemented placeholders (YAGNI).** `standardFilter` (project-list
  filtering) and `routingSettings` (OpenRouteService) were reserved for
  features that were never built and are not read anywhere.

A codebase-wide search confirmed none of these fields are read or written
outside their own struct definitions, tests, and the generated TypeScript
bindings.

### Evaluated Options

- Remove the dead fields (`tokenReferences`, `contactFilter`, `standardFilter`,
  `routingSettings`) and keep `planradarBaseUrl`
  - Pros: Schema reflects what the app actually uses; no stale duplicate of the
    keychain token state; less surface to keep consistent across Rust, generated
    bindings, and tests; backward-compatible on read because serde ignores
    unknown keys.
  - Cons: A future feature that needs one of these must reintroduce it; one more
    schema-changing ADR to track.
- Leave the fields in place
  - Pros: No change; any future feature finds its slot already present.
  - Cons: Misleading schema (e.g. a `tokenReferences` that is always empty while
    real tokens live in the keychain); ongoing maintenance of fields nothing
    reads; violates YAGNI.
- Remove everything unused including `planradarBaseUrl`
  - Pros: Smallest possible schema.
  - Cons: PlanRadar is an actively planned integration in the OpenSpec backlog;
    its base URL is genuine configuration, not dead token or filter state, so
    removing it would churn the schema again once that work starts.

## Decision

Remove the following from `LocalStore` and the generated bindings:

- `tokenReferences` (entire struct) — superseded by keychain storage.
- `contactFilter` — superseded by the fixed category search (ADR 0012).
- `standardFilter` — unused.
- `routingSettings` — unused.

Keep `apiEndpoints.planradarBaseUrl`: PlanRadar is a planned integration
(see OpenSpec backlog) and the endpoint is configuration, not dead token or
filter state.

The remaining store schema is: `apiEndpoints`, `employeeSettings`,
`displaySettings`, `dayliteCache`, and `holidayCache`.

## Consequences

- The on-disk format shrinks. Removal is backward-compatible on read: serde
  ignores unknown keys, so existing `local-store.json` files that still contain
  the removed fields keep loading. New writes simply omit them.
- This supersedes the persistence detail of ADR 0006 (tokens are in the
  keychain, not `tokenReferences`) and narrows the schema listed in ADR 0005.
- Future filtering or routing features should reintroduce only the fields they
  actually consume, rather than carrying speculative configuration.
