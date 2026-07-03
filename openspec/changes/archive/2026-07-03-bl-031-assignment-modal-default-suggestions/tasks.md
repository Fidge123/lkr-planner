## 1. Overdue Project Query (Rust, TDD)

- [x] 1.1 Write failing test: overdue query sends `{"category": {"equal": "Überfällig"}}` paired with status `new_status` / `in_progress` as OR clauses (single call)
- [x] 1.2 Implement `daylite_query_overdue_projects` Tauri command using BL-022 `DayliteSearchInput` infrastructure
- [x] 1.3 Write failing test: overdue results are sorted by numeric ID and limited to 5
- [x] 1.4 Add VCR cassette for overdue project query

## 2. Suggestion Logic

- [x] 2.1 Implement client last-used cache that records the last assigned project (in-memory, session-scoped)
- [x] 2.2 Combine overdue results with the cached recent project first
- [x] 2.3 Deduplicate the recent project out of the overdue portion
- [x] 2.4 Cap total suggestions at 5 (recent first if present, otherwise up to 5 overdue)

## 3. UI Implementation

- [x] 3.1 Feed suggestions into BL-032's combobox empty-state list
- [x] 3.2 Render suggestions as clickable items
- [x] 3.3 Implement click handler to select suggestion into the assignment field
- [x] 3.4 Restore suggestions when the filter is cleared or Escape resets a non-empty filter
- [x] 3.5 Ensure keyboard nav (arrow/Enter) operates over suggestions via BL-032's mechanism
- [x] 3.6 Show German empty-state message when no suggestions

## 4. Fallback Handling

- [x] 4.1 Handle case with no recent assignment
- [x] 4.2 Handle case with no overdue projects
- [x] 4.3 Handle case with neither available

## 5. Testing

- [x] 5.1 Write UI tests for suggestion ordering (recent first, then overdue)
- [x] 5.2 Write UI tests for suggestion count limit (max 5)
- [x] 5.3 Write test for dedup: recent project that is also overdue appears once
- [x] 5.4 Write test for empty cache: shows up to 5 overdue projects
- [x] 5.5 Write service tests for fallback behavior
- [x] 5.6 Write tests for empty state message display
- [x] 5.7 Write test: clearing the filter / Escape restores the default suggestions
