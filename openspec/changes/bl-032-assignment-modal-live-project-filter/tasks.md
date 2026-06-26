## 1. Sort Infrastructure (Rust, TDD)

- [x] 1.1 Write failing test: `search_projects_core` sorts by name when sort = name
- [x] 1.2 Add optional sort field to `DayliteSearchInput` (default numeric ID)
- [x] 1.3 Implement name sort using Rust default ordering (locale-aware ä/ö/ü only if trivial); keep ID as default
- [x] 1.4 Confirm existing callers default to ID sort (BL-022 contract unchanged)

## 2. Trailing Debounce Hook

- [x] 2.1 Add new trailing-edge debounce hook for search-as-you-type (300ms)
- [x] 2.2 Add request-sequence guard so stale responses are dropped

## 3. Combobox Shell

- [x] 3.1 Replace the `<select>` with a filter input + result list
- [x] 3.2 Remove the bulk `loadProjectsForAssignmentPicker` pre-load
- [x] 3.3 Leave the result list empty when the filter is empty

## 4. Live Filtering

- [x] 4.1 Query only when the filter has at least 3 characters (sort = name)
- [x] 4.2 Show filtered results in the result list
- [x] 4.3 Limit results to 5
- [x] 4.4 Filter to new_status and in_progress only

## 5. Keyboard Navigation

- [x] 5.1 Add arrow key handlers operating on the displayed list (generic, covers BL-031 content)
- [x] 5.2 Add Enter key to select highlighted project into the assignment field
- [x] 5.3 Intercept Escape on keydown: clear non-empty filter, else fall through to modal close

## 6. Testing

- [x] 6.1 Write UI tests for filter → results and clear → empty default state
- [x] 6.2 Write service tests for max-5, status filter, and name-sort guarantees
- [x] 6.3 Write UI tests for keyboard selection
- [x] 6.4 Write tests for trailing debounce and stale-response dropping
- [x] 6.5 Write tests for Escape precedence (clear vs close)