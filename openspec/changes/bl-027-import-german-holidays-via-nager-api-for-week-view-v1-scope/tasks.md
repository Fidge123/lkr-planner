## 1. Setup and API Client

- [ ] 1.1 Add reqwest dependency for HTTP requests
- [ ] 1.2 Create German holiday import module
- [ ] 1.3 Define Holiday struct with date, name, and state fields
- [ ] 1.4 Implement Nager API client for Germany

## 2. Holiday Fetching

- [ ] 2.1 Implement fetch_holidays(year) method
- [ ] 2.2 Filter for global and MV-only holidays
- [ ] 2.3 Handle year-boundary weeks (fetch both years)
- [ ] 2.4 Cache holiday data in memory by year

## 3. Error Handling

- [ ] 3.1 Implement timeout handling (5s)
- [ ] 3.2 Add German error message for API failures
- [ ] 3.3 Graceful degradation (show warning, no crash)

## 4. Testing

- [ ] 4.1 Write service tests for Nager API mapping (DE, global, MV)
- [ ] 4.2 Write tests verifying non-MV state holidays are excluded
- [ ] 4.3 Write tests for year-boundary fetch
- [ ] 4.4 Write tests for timeout and failure behavior