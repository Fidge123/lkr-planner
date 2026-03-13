## 1. Filter Input

- [ ] 1.1 Add text input field above suggestions
- [ ] 1.2 Implement debounce (300ms) on input
- [ ] 1.3 Clear filter restores default suggestions

## 2. Live Filtering

- [ ] 2.1 Call BL-022 query with filter text
- [ ] 2.2 Replace default suggestions with filtered results
- [ ] 2.3 Limit results to 5
- [ ] 2.4 Filter to new_status and in_progress only

## 3. Keyboard Navigation

- [ ] 3.1 Add arrow key handlers for list navigation
- [ ] 3.2 Add Enter key to confirm selection
- [ ] 3.3 Add Escape key to clear filter

## 4. Testing

- [ ] 4.1 Write UI tests for replace/restore behavior on input change
- [ ] 4.2 Write service tests for max-5 and status filter guarantees
- [ ] 4.3 Write UI tests for keyboard selection
- [ ] 4.4 Write tests for debounce behavior