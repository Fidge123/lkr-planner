## 1. Timeout Handling (Rust, TDD)

- [x] 1.1 Write failing test: reqwest timeout maps to `DayliteApiErrorCode::Timeout`
- [x] 1.2 Add `Timeout` variant to `DayliteApiErrorCode` enum
- [x] 1.3 Add 5s timeout to reqwest client builder in `ReqwestTransport`
- [x] 1.4 Map reqwest timeout error to `Timeout` code with German message `"Zeitüberschreitung bei der Daylite-Anfrage"`

## 2. Status Filter in Search (Rust, TDD)

- [x] 2.1 Write failing test: search with multiple statuses sends array body with one OR-clause per status
- [x] 2.2 Add `statuses: Option<Vec<String>>` field to `DayliteSearchInput`
- [x] 2.3 Update `search_projects_core` to build array body when statuses provided, plain object body otherwise
- [x] 2.4 Write failing test: search without status filter sends plain object body (backwards-compatible)
- [x] 2.5 Regenerate TypeScript bindings for updated `DayliteSearchInput`

## 3. Deterministic Ordering (Rust, TDD)

- [x] 3.1 Write failing test: results with mixed numeric IDs sort numerically ascending
- [x] 3.2 Implement numeric ID extraction and sort in `search_projects_core`
- [x] 3.3 Write failing test: limit is applied after sort

## 4. Error Message Normalization (Rust, TDD)

- [x] 4.1 Write failing test: malformed response returns `InvalidResponse` with German message `"Ungültige Antwort von Daylite"`
- [x] 4.2 Write failing test: timeout returns `Timeout` code with correct German message

## 5. Integration Testing

- [ ] 5.1 Update VCR cassette for `daylite-search-projects` to include status-filtered request
- [ ] 5.2 Write VCR replay test verifying status filter produces only `new_status`/`in_progress` results
