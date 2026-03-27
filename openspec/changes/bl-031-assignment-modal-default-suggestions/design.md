## Context

The assignment modal needs default suggestions to help users quickly select projects. Suggestions are shown when the modal first opens before any user filtering.

## Goals / Non-Goals

**Goals:**
- Show deterministic suggestions in consistent order
- Prioritize most recently assigned project first
- Show overdue projects as secondary suggestions
- Handle empty states gracefully with German messages

**Non-Goals:**
- Free-text filtering behavior (covered by BL-032)
- Persisting personal search history

## Decisions

### Suggestion Ordering
**Decision**: Most recently assigned project first, then overdue projects
- Query assignment history for most recent project
- Query overdue projects via BL-022 query
- Combine results, cap at 5 total suggestions

### Fallback Behavior
**Decision**: Show empty state message when no suggestions available
- If no recent assignment AND no overdue projects
- Show "Keine Vorschläge verfügbar" in German

### Determinism
**Decision**: Use consistent ordering for identical states
- Sort by assignment date descending for recent
- Sort by project ID for overdue (consistent tie-breaking)

## Risks / Trade-offs

- **Risk**: Slow query affecting modal open time
  - **Mitigation**: Cache recent assignment query result

- **Risk**: Many overdue projects
  - **Mitigation**: Strict limit of 5 suggestions total
