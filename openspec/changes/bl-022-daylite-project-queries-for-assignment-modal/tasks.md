## 1. Setup and Types

- [ ] 1.1 Create Daylite project query module structure
- [ ] 1.2 Define ProjectQueryFilter struct with status and limit fields
- [ ] 1.3 Define ProjectSearchResult type

## 2. Query Implementation

- [ ] 2.1 Implement basic project search with text query
- [ ] 2.2 Add status filtering (new_status, in_progress only)
- [ ] 2.3 Add result limiting (limit=5)
- [ ] 2.4 Add sorting by ID ascending for determinism

## 3. Overdue Query

- [ ] 3.1 Implement overdue project query method
- [ ] 3.2 Return top 5 overdue projects sorted by ID

## 4. Error Handling

- [ ] 4.1 Define DayliteQueryError enum
- [ ] 4.2 Implement timeout error handling
- [ ] 4.3 Implement malformed response error handling
- [ ] 4.4 Add German error messages

## 5. Testing

- [ ] 5.1 Write unit tests for search filtering (new_status, in_progress)
- [ ] 5.2 Write unit tests for result limiting behavior
- [ ] 5.3 Write unit tests for deterministic ordering
- [ ] 5.4 Write unit tests for overdue query
- [ ] 5.5 Write tests for timeout error handling
- [ ] 5.6 Write tests for malformed payload error handling
