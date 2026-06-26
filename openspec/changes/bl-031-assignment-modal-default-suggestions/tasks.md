## 1. Overdue Project Query (Rust, TDD)

- [ ] 1.1 Write failing test: overdue query sends `{"category": {"equal": "Überfällig"}}` in body (single call, no status filter)
- [ ] 1.2 Implement `daylite_query_overdue_projects` Tauri command using BL-022 `DayliteSearchInput` infrastructure
- [ ] 1.3 Write failing test: overdue results are sorted by numeric ID and limited to 5
- [ ] 1.4 Add VCR cassette for overdue project query

## 2. Suggestion Logic

- [ ] 2.1 Implement client last-used cache that records the last assigned project (in-memory, session-scoped)
- [ ] 2.2 Combine overdue results with the cached recent project first
- [ ] 2.3 Deduplicate the recent project out of the overdue portion
- [ ] 2.4 Cap total suggestions at 5 (recent first if present, otherwise up to 5 overdue)

## 3. UI Implementation

- [ ] 3.1 Feed suggestions into BL-032's combobox empty-state list
- [ ] 3.2 Render suggestions as clickable items
- [ ] 3.3 Implement click handler to select suggestion into the assignment field
- [ ] 3.4 Restore suggestions when the filter is cleared or Escape resets a non-empty filter
- [ ] 3.5 Ensure keyboard nav (arrow/Enter) operates over suggestions via BL-032's mechanism
- [ ] 3.6 Show German empty-state message when no suggestions

## 4. Fallback Handling

- [ ] 4.1 Handle case with no recent assignment
- [ ] 4.2 Handle case with no overdue projects
- [ ] 4.3 Handle case with neither available

## 5. Testing

- [ ] 5.1 Write UI tests for suggestion ordering (recent first, then overdue)
- [ ] 5.2 Write UI tests for suggestion count limit (max 5)
- [ ] 5.3 Write test for dedup: recent project that is also overdue appears once
- [ ] 5.4 Write test for empty cache: shows up to 5 overdue projects
- [ ] 5.5 Write service tests for fallback behavior
- [ ] 5.6 Write tests for empty state message display
- [ ] 5.7 Write test: clearing the filter / Escape restores the default suggestions
