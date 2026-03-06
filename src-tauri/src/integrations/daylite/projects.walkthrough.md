# Walkthrough: `src-tauri/src/integrations/daylite/projects.rs`

## Purpose

This file handles Daylite project listing and project search. It translates raw Daylite project payloads into normalized planning records and persists any refreshed auth state after requests succeed.

## Block by block

### Imports (`lines 1-12`)

- The file uses the shared authenticated request helper, the Daylite client, query-building helpers, and `chrono` for date normalization.
- `serde_json::json` is used to build request bodies inline.

### Project models (`lines 14-69`)

- `DayliteProjectSummary` matches the Daylite API response. The `alias` attributes accept alternate field names such as `create_date` and `modify_date`.
- `PlanningProjectStatus` is the app's normalized status enum.
- `PlanningProjectRecord` is the cleaned-up record returned to the frontend.

Rust syntax to notice:
- `Option<String>` is used for fields that may be absent in the API response.
- `skip_serializing_if` keeps outbound JSON compact when optional fields are missing.

Best practice:
- Introduce a normalized enum when third-party status values are inconsistent or loosely controlled.

### Public Tauri commands (`lines 71-101`)

- `daylite_list_projects` loads the store, performs the request, persists refreshed tokens, and returns normalized project records.
- `daylite_search_projects` does the same but returns a paginated `DayliteSearchResult<DayliteProjectSummary>` because the frontend still wants raw search results and pagination metadata.

Best practice:
- Persist refreshed tokens immediately after the request that produced them.

### Core request helpers (`lines 103-154`)

- `list_projects_core` sends `POST /projects/_search` with `full-records=true` and an empty JSON object to fetch project records, then maps them into planning records.
- `search_projects_core` sends the same endpoint with a name filter and optional `limit` query parameter.
- Both functions return the refreshed `DayliteTokenState` alongside the payload.

Rust syntax to notice:
- `json!({})` is a convenient way to create a JSON object literal in Rust.
- The multiline return type of `search_projects_core` is still just one `Result<(payload, token_state), DayliteApiError>`.

### Mapping and normalization (`lines 156-245`)

- `map_daylite_project_summary` trims and normalizes every relevant field.
- `map_project_status` maps Daylite strings onto the internal enum and defaults unknown values to `NewStatus`.
- `normalize_reference`, `normalize_optional_string`, and `normalize_keywords` remove surrounding whitespace and discard empty values.
- `normalize_optional_date` accepts either RFC 3339 timestamps or plain `YYYY-MM-DD` dates and converts them to UTC ISO 8601 with millisecond precision.

Rust syntax to notice:
- `?` is used with `Option` inside `normalize_optional_date`; in that context it exits early with `None`.
- `DateTime::parse_from_rfc3339` and `NaiveDate::parse_from_str` show a common "try one format, then another" parsing pattern.

Best practice:
- Normalize dates at the backend boundary so the frontend can treat them consistently.

### Tests (`lines 247-524`)

- The tests cover status mapping, project normalization, list and search request shape, token refresh behavior, and VCR replay.
- `MockTransport` again records sent requests and serves queued responses.

Rust syntax to notice:
- The tests assert request query/body details, not only final mapped results, which helps catch accidental API contract regressions.

## Best practices this file demonstrates

- Separate raw search DTOs from normalized planning records.
- Keep date normalization close to the backend edge.
- Verify both outbound request shape and inbound mapping logic in tests.
