## Why

The assignment modal needs to query Daylite projects for user selection. This requires a dedicated query API that filters by project status, supports text search, and provides deterministic results for modal usage.

## What Changes

- Implement Daylite project query service with status filtering (new/in_progress only)
- Add text search support (project name, external reference)
- Add deterministic result limiting (limit=5)
- Add overdue project query for default suggestions
- Normalize API errors into German user-facing messages

## Capabilities

### New Capabilities
- `daylite-project-query`: Query service for Daylite projects used by assignment modal

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New Rust module for Daylite project queries in Tauri backend
- APIs: Daylite MarketKit API integration
- Dependencies: Depends on existing Daylite project read/search command foundation (BL-006)
