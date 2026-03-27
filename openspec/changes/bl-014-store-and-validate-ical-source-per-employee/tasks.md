## 1. URL Validation

- [ ] 1.1 Implement URL format validation (HTTP/HTTPS check)
- [ ] 1.2 Reject invalid URLs before network call

## 2. Connection Testing

- [ ] 2.1 Implement HTTP request for iCal URL
- [ ] 2.2 Add independent test for primary iCal
- [ ] 2.3 Add independent test for absence iCal
- [ ] 2.4 Parse response to verify iCal content

## 3. German Error Messages

- [ ] 3.1 Map connection timeout to German message
- [ ] 3.2 Map SSL/certificate errors to German message
- [ ] 3.3 Map invalid response to German message
- [ ] 3.4 Add actionable hints for each error type

## 4. Timestamp Persistence

- [ ] 4.1 Store test timestamp after each test
- [ ] 4.2 Make timestamp available for UI display

## 5. Testing

- [ ] 5.1 Validation tests for allowed/disallowed URL formats
- [ ] 5.2 Service tests for independent primary vs absence test execution
- [ ] 5.3 UI tests for result rendering and retry behavior