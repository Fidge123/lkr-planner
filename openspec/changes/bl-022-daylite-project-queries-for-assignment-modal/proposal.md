## Why

The assignment modal needs to query Daylite projects for user selection. This requires enhancing the existing project search with status filtering, deterministic ordering, timeout handling, and German error messages.

## What Changes

- Enhance existing `daylite_search_projects` command with optional status filter parameter
- Add server-side status filtering (new_status / in_progress) to search body — consistent with how contacts use `{"category": {"equal": "Monteur"}}`
- Add deterministic result limiting (limit=5) with numeric ID sort in Rust
- Add `Timeout` error code variant to `DayliteApiErrorCode`
- Normalize API errors into German user-facing messages

## Capabilities

### New Capabilities
- `daylite-project-query`: Enhanced project search with status filter, timeout handling, and deterministic ordering

### Modified Capabilities
- `daylite_search_projects`: Extended with optional status filter

## Out of Scope

- Overdue project query → deferred to BL-031
- Text search by external reference → removed; name search only

## Impact

- Code: Modify existing `projects.rs` and `shared.rs` in Rust backend
- APIs: Daylite MarketKit API — server-side status filter in search body
- Dependencies: Depends on existing Daylite project read/search command foundation (BL-006)
