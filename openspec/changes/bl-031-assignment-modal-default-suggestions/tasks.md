## 1. Suggestion Logic

- [ ] 1.1 Implement query for most recently assigned project
- [ ] 1.2 Implement query for overdue projects (via BL-022)
- [ ] 1.3 Combine results with recent project first
- [ ] 1.4 Cap total suggestions at 5

## 2. UI Implementation

- [ ] 2.1 Add suggestions section to assignment modal
- [ ] 2.2 Render suggestions as clickable items
- [ ] 2.3 Implement click handler to select suggestion
- [ ] 2.4 Show German empty-state message when no suggestions

## 3. Fallback Handling

- [ ] 3.1 Handle case with no recent assignment
- [ ] 3.2 Handle case with no overdue projects
- [ ] 3.3 Handle case with neither available

## 4. Testing

- [ ] 4.1 Write UI tests for suggestion ordering
- [ ] 4.2 Write UI tests for suggestion count limit (max 5)
- [ ] 4.3 Write service tests for fallback behavior
- [ ] 4.4 Write tests for empty state message display