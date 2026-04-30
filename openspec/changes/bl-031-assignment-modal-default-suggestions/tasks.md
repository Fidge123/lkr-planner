## 1. Overdue Project Query (Rust, TDD)

- [ ] 1.1 Write failing test: overdue query sends `{"category": {"equal": "Überfällig"}}` and active status filter in body
- [ ] 1.2 Implement `daylite_query_overdue_projects` Tauri command using BL-022 `DayliteSearchInput` infrastructure
- [ ] 1.3 Write failing test: overdue results are sorted by numeric ID and limited to 5
- [ ] 1.4 Add VCR cassette for overdue project query

## 2. Suggestion Logic

- [ ] 2.1 Implement query for most recently assigned project
- [ ] 2.2 Combine overdue results with recent project first
- [ ] 2.3 Cap total suggestions at 5

## 3. UI Implementation

- [ ] 3.1 Add suggestions section to assignment modal
- [ ] 3.2 Render suggestions as clickable items
- [ ] 3.3 Implement click handler to select suggestion
- [ ] 3.4 Show German empty-state message when no suggestions

## 4. Fallback Handling

- [ ] 4.1 Handle case with no recent assignment
- [ ] 4.2 Handle case with no overdue projects
- [ ] 4.3 Handle case with neither available

## 5. Testing

- [ ] 5.1 Write UI tests for suggestion ordering
- [ ] 5.2 Write UI tests for suggestion count limit (max 5)
- [ ] 5.3 Write service tests for fallback behavior
- [ ] 5.4 Write tests for empty state message display
