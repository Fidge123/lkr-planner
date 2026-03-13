## Context

The assignment modal needs live filtering as the user types. This provides instant feedback and quick project selection without page navigation.

## Goals / Non-Goals

**Goals:**
- Filter projects as user types
- Show filtered results instead of default suggestions while filtering
- Support keyboard navigation in filtered list
- Restore default suggestions when filter is cleared

**Non-Goals:**
- Persisting personal search history
- Advanced search operators

## Decisions

### Filter Trigger
**Decision**: Show filtered results when filter input has at least 1 character
- Debounce input to avoid excessive queries (300ms)
- Clear filter restores default suggestions

### Results Display
**Decision**: Replace default suggestions with filtered list
- Show up to 5 matching projects
- Filter to only new_status and in_progress states
- Sort by project name for consistent results

### Keyboard Navigation
**Decision**: Support arrow keys and Enter for selection
- Up/Down to navigate filtered list
- Enter to select highlighted project
- Escape to clear filter and restore defaults

## Risks / Trade-offs

- **Risk**: Many projects matching filter
  - **Mitigation**: Strict limit of 5 results

- **Risk**: Rapid typing causing race conditions
  - **Mitigation**: Debounce and cancel pending requests
