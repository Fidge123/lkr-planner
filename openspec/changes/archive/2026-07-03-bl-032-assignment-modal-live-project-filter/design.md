## Context

The assignment modal needs live filtering as the user types. This provides instant feedback and quick project selection without page navigation.

## Goals / Non-Goals

**Goals:**
- Build the combobox shell (filter input + result list + keyboard navigation) that replaces today's `<select>`
- Filter projects as user types
- Support keyboard navigation over the displayed result list
- Leave the result list empty when the filter is empty (a self-contained empty default state)

**Non-Goals:**
- Default suggestion content for the empty state (recent + overdue) — owned by `assignment-modal-suggestions` / BL-031
- Persisting personal search history
- Advanced search operators

## Decisions

### Independence from default suggestions
**Decision**: bl-032 ships first with a generic empty default state, no dependency on BL-031
- The result list renders whatever items it is given; when the filter is empty it renders the empty-state content (nothing until BL-031 plugs in recent + overdue)
- All default-suggestion behavior, including restoring suggestions when the filter clears or Escape is pressed, lives in BL-031
- This breaks the previous circular reference: BL-031 depends on this combobox shell, this change depends on nothing from BL-031

### Filter Trigger
**Decision**: Show filtered results when filter input has at least 3 characters
- Below 3 characters no query is sent and the list stays in its empty default state
- Debounce input to avoid excessive queries (300ms)
- Clearing the filter returns the list to its empty default state

### Trailing debounce hook
**Decision**: Add a new trailing-edge debounce hook for search-as-you-type
- The existing `useLeadingDebounce` fires on the first keystroke (right for week navigation, wrong for a 1-char search that would query instantly)
- New hook waits until typing settles (300ms) before emitting, so queries fire on the settled term
- Pair it with a request-sequence guard (monotonic request id, like `usePlanningAssignments`) so a slow earlier query cannot overwrite a newer result

### Results Display
**Decision**: Show filtered list in the result area
- Show up to 5 matching projects
- Filter to only new_status and in_progress states
- Sort by project name

### Sorting infrastructure
**Decision**: Add optional sort support to `search_projects_core`; default stays numeric ID
- `DayliteSearchInput` gains an optional sort field (e.g. `id` default | `name`)
- Numeric-ID sort remains the default for all existing callers (BL-022 contract unchanged)
- bl-032 passes sort = name; BL-031's overdue query keeps the default ID sort
- Name sort uses Rust's default string ordering (byte order); German locale-aware collation (ä/ö/ü) is only worth adding if it turns out to be very simple, otherwise deferred

### Keyboard Navigation
**Decision**: Support arrow keys and Enter over whichever list is displayed
- Up/Down navigate the currently displayed items (filtered results or, once BL-031 lands, default suggestions)
- Navigation operates on the unified list structure, so it covers BL-031 content automatically
- Enter selects the highlighted project into the assignment field and leaves the modal open; the user confirms with Speichern (no save-and-close shortcut for now, can be added later if needed)
- Escape clears a non-empty filter (returns to empty default state); on an empty filter it falls through to the modal close flow

### Escape precedence
**Decision**: Intercept Escape on `keydown` before the native `<dialog>` cancel event
- The modal's native `cancel` event currently maps Escape → close; it has no knowledge of filter state
- The combobox handles `keydown` and calls `preventDefault` when the filter is non-empty (clear instead of close), otherwise lets the close flow proceed

### Removal of the bulk project pre-load
**Decision**: This change removes the `<select>` and its bulk `loadProjectsForAssignmentPicker` pre-load
- The combobox queries per settled keystroke instead of pre-loading all active projects
- Note the interim gap: between bl-032 shipping and BL-031 landing, an empty filter shows an empty list, so the user must type to pick a project (no full-list browsing)

## Risks / Trade-offs

- **Risk**: Many projects matching filter
  - **Mitigation**: Strict limit of 5 results

- **Risk**: Rapid typing causing race conditions
  - **Mitigation**: Trailing debounce plus request-sequence guard that drops stale responses

- **Risk**: Interim regression — no full-project browsing until BL-031 lands
  - **Mitigation**: Sequence BL-031 close behind bl-032, or accept type-to-find as the interim behavior
