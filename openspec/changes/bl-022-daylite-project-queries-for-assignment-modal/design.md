## Context

The assignment modal requires Daylite project queries with specific filtering behavior. The modal needs to display a limited set of projects for selection, with support for text search and default suggestions for overdue projects.

## Goals / Non-Goals

**Goals:**
- Provide project search with status filtering (new/in_progress only)
- Support text search by project name and external reference
- Return deterministic first 5 results for identical input
- Provide overdue project query for default suggestions
- Normalize errors into German user-facing messages

**Non-Goals:**
- Modal UI behavior and suggestion ordering logic
- Bulk operations
- Project creation/modification through this query service

## Decisions

### Search Implementation
**Decision**: Use Daylite's MarketKit search with custom result filtering
- Daylite provides project search via MarketKit
- Filter results in Rust to ensure only new_status and in_progress projects
- Apply limit before returning results for deterministic ordering

### Result Determinism
**Decision**: Sort by project ID ascending before applying limit
- Ensures identical input returns identical results
- Simple and predictable ordering

### Error Handling
**Decision**: Map Daylite errors to internal error enum with German messages
- Translate common errors to user-friendly German text
- Include original error for debugging

## Risks / Trade-offs

- **Risk**: Slow search response affecting modal UX
  - **Mitigation**: Implement timeout (5s) and show loading state

- **Risk**: Many projects causing slow queries
  - **Mitigation**: Server-side limit when possible, client-side fallback
