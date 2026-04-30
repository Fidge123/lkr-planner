## Context

The assignment modal requires Daylite project search with status filtering, deterministic ordering, and timeout handling. This change enhances the existing `daylite_search_projects` command rather than introducing a new module, keeping it reusable for future consumers.

## Goals / Non-Goals

**Goals:**
- Server-side status filtering (new_status / in_progress) in search body
- Text search by project name
- Return deterministic first 5 results for identical input
- Timeout handling with German error message
- Normalize errors into German user-facing messages

**Non-Goals:**
- Overdue project query (moved to BL-031)
- Text search by external reference (removed)
- Modal UI behavior and suggestion ordering logic
- Bulk operations or project creation/modification

## Decisions

### Search Implementation
**Decision**: Server-side status filtering via Daylite API search body
- Contacts use `{"category": {"equal": "Monteur"}}` — projects must use the same pattern
- Current project search sends no status filter and returns all statuses — this is the gap to fix
- For two statuses (`new_status`, `in_progress`): if the API supports an `"in"` operator, use a single call; otherwise make two `{"equal": "..."}` calls and merge results
- Rust handles deduplication and applies limit after merge

### Result Determinism
**Decision**: Numeric sort by project ID in Rust before applying limit
- Project IDs are paths like `/v1/projects/3001` — extract the trailing integer for comparison
- String sort would give wrong order (`/v1/projects/100` before `/v1/projects/20`)
- Sort ascending by numeric ID, then apply limit=5

### Timeout Handling
**Decision**: Add `Timeout` variant to `DayliteApiErrorCode`
- Add 5s timeout to the reqwest client builder
- Map reqwest timeout errors to the new `Timeout` code
- German user message: `"Zeitüberschreitung bei der Daylite-Anfrage"`
- Frontend can distinguish timeout from other failures

### Error Normalization
**Decision**: Map Daylite errors to `DayliteApiError` with German `user_message`
- Reuse existing `DayliteApiError` structure
- Add `Timeout` to `DayliteApiErrorCode` enum
- Malformed response maps to existing `InvalidResponse` code with message `"Ungültige Antwort von Daylite"`

### Command Design
**Decision**: Extend existing `daylite_search_projects` with optional status filter
- Add `statuses: Option<Vec<String>>` to `DayliteSearchInput`
- When statuses provided, include in Daylite search body
- Backwards-compatible: existing callers that pass no statuses continue to receive all statuses

## Risks / Trade-offs

- **Risk**: Daylite API may not support multi-value status filter in a single call
  - **Mitigation**: Fall back to two sequential calls merged in Rust; document in code

- **Risk**: Slow search response affecting modal UX
  - **Mitigation**: 5s timeout; German error shown in modal

- **Risk**: Many projects matching filter
  - **Mitigation**: Numeric sort + limit=5 applied after any merge
