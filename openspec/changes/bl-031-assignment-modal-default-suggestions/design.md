## Context

The assignment modal needs default suggestions to help users quickly select projects. Suggestions are shown when the modal first opens before any user filtering.

## Goals / Non-Goals

**Goals:**
- Show deterministic suggestions in consistent order
- Prioritize most recently assigned project first
- Show overdue projects as secondary suggestions
- Handle empty states gracefully with German messages

**Non-Goals:**
- Free-text filtering behavior (covered by BL-032, which also provides the combobox shell this change renders suggestions into)
- Persisting personal search history across sessions (the last-used cache is in-memory and resets on restart)

## Decisions

### Overdue Project Query
**Decision**: Query Daylite by category `"Überfällig"` only — no additional status filter
- Use `{"category": {"equal": "Überfällig"}}` in Daylite search body as a single call
- No separate status filter: the Daylite API has no `in` operator for scalar fields, so filtering two statuses requires two calls; projects in the "Überfällig" category are by definition active (done/abandoned projects are not marked overdue)
- This is a new Tauri command added in this change, building on BL-022 infrastructure
- Sort by numeric project ID ascending (same as BL-022), limit to 5

### Most Recently Assigned Project Source
**Decision**: Derive "most recently assigned" from a client-side last-used cache, not from CalDAV history
- Assignments live as CalDAV VEVENTs loaded per week per employee; there is no cross-time assignment history query, and the iCal parser does not retain `CREATED`/`LAST-MODIFIED`
- Instead, remember the last project the user assigned during the current session in a temporary client cache (in-memory, reset on app restart)
- This avoids a slow multi-calendar CalDAV scan and keeps the feature deterministic for a given cache state
- Trade-off: "recent" is session-scoped, so a freshly started session shows overdue-only suggestions until the user makes an assignment

### Suggestion Ordering
**Decision**: Recent project first (if cached), then overdue projects
- Read the most recent project from the client last-used cache
- Query overdue projects via the new overdue command
- Combine results, cap at 5 total suggestions
- When the cache is empty, show up to 5 overdue projects

### Deduplication
**Decision**: A project appears at most once across the combined list
- If the recent project is also in the overdue results, keep it only in the first (recent) position and drop it from the overdue portion
- Dedup happens before the cap of 5 is applied, so the list holds 5 distinct projects when that many exist

### Fallback Behavior
**Decision**: Show empty state message when no suggestions available
- If the client last-used cache is empty AND no overdue projects exist
- Show "Keine Vorschläge verfügbar" in German

### Determinism
**Decision**: Use consistent ordering for identical states
- Recent project comes directly from the client last-used cache
- Sort overdue by project ID for consistent tie-breaking

### Empty-state ownership (relationship to BL-032)
**Decision**: All default-suggestion behavior lives here, plugged into BL-032's combobox empty state
- BL-032 ships the combobox shell (input + result list + keyboard nav) with a generic empty state
- This change supplies the empty-state content (recent + overdue) and the behavior of restoring it when the filter is cleared or Escape resets a non-empty filter
- Keyboard navigation is BL-032's mechanism operating on the displayed list; this change only ensures suggestions feed into that same list structure so arrow/Enter work over them
- This change therefore depends on BL-032; BL-032 does not depend on it

## Risks / Trade-offs

- **Risk**: Slow query affecting modal open time
  - **Mitigation**: Recent project comes from an in-memory cache (no query); only the overdue query hits the network

- **Risk**: Many overdue projects
  - **Mitigation**: Strict limit of 5 suggestions total
