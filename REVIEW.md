# Rust File Walkthrough (`main...daylite`)

This guide is organized per file, not per commit.  
For each function: what it does, Rust syntax notes, and potential improvements/unusual details.

Conventions used below:
- `Result<T, E>` = success/error return, often paired with `?` for early return.
- `Option<T>` = value may be absent (`Some`) or missing (`None`).
- `#[tauri::command]` + `#[specta::specta]` = frontend-callable API plus TS type export metadata.

---

## File: `src-tauri/src/lib.rs`

### `run`
- What: Builds and configures the Tauri app, registers all command handlers, exports Specta TypeScript bindings, installs plugins.
- Syntax: `tauri_specta::collect_commands![...]` is a macro that gathers function paths at compile time.
- Improvement/unusual:
  - `update(handle).await.unwrap()` in spawned task can panic; returning/logging a handled error would be safer in production.

### `update`
- What: Checks for app updates and performs download/install with progress callbacks.
- Syntax: `if let Some(update) = app.updater()?.check().await?` combines optional and fallible async flow.
- Improvement/unusual:
  - Uses `println!` for progress; consider structured logging so logs are easier to filter.

---

## File: `src-tauri/src/integrations/mod.rs`

No functions in this file.  
It only exposes modules (`daylite`, `health`, `local_store`).

---

## File: `src-tauri/src/integrations/local_store.rs`

### `LocalStore::default`
- What: Defines default full store contents when no config exists.
- Syntax: `impl Default for LocalStore` is Rust's standard default-constructor pattern.
- Improvement/unusual:
  - Good baseline; defaults are explicit and readable.

### `StandardFilter::default`
- What: Provides domain defaults for planning filter fields.
- Syntax: `vec![...]` macro creates vectors inline.
- Improvement/unusual:
  - Hard-coded German strings are okay for domain config but could be centralized if reused elsewhere.

### `ContactFilter::default`
- What: Sets default active employee keyword (`"Monteur"`).
- Syntax: Small `Default` impl returning a struct literal.
- Improvement/unusual:
  - If this keyword changes often, making it a constant could improve discoverability.

### `RoutingSettings::default`
- What: Sets default routing profile and empty API key.
- Syntax: Struct literal with owned `String` values.
- Improvement/unusual:
  - Sensible default.

### `load_local_store`
- What: Resolves app config path and loads the JSON store file.
- Syntax: Chained `map(...).map_err(...)` on `Result`.
- Improvement/unusual:
  - Clear error mapping to user vs technical messages.

### `save_local_store`
- What: Resolves app config path and writes the JSON store file.
- Syntax: Similar to loader; wraps lower-level writer helper.
- Improvement/unusual:
  - Clear separation of path resolution and actual I/O.

### `load_store_from_path`
- What: Core loader; returns defaults for missing file, otherwise parses JSON into `LocalStore`.
- Syntax:
  - `if !path.exists() { return Ok(LocalStore::default()); }`
  - `serde_json::from_str::<LocalStore>(&content)` for typed deserialization.
- Improvement/unusual:
  - Detecting "missing field" via string matching on error text is brittle. Pattern-matching `serde_json::Error` categories would be stronger.

### `save_store_to_path`
- What: Ensures parent directory exists, serializes pretty JSON, writes to disk.
- Syntax: `if let Some(parent) = path.parent() { ... }` for optional parent handling.
- Improvement/unusual:
  - Pretty JSON is human-friendly; if write size becomes concern, compact serialization is an alternative.

### Test: `loads_defaults_for_missing_store_file`
- What: Verifies missing store path returns defaults.
- Syntax: standard `#[test]`.
- Improvement/unusual: Good coverage.

### Test: `saves_and_loads_store_restart_safe`
- What: Round-trip test for complete store content.
- Syntax: equality assertion on entire struct graph (`assert_eq!`).
- Improvement/unusual: Strong regression test.

### Test: `returns_german_error_with_technical_details_for_corrupt_json`
- What: Verifies corrupt JSON surfaces expected error code/messages.
- Syntax: `expect_err` for negative path assertions.
- Improvement/unusual: Good UX contract test.

### Test: `returns_german_error_with_technical_details_for_missing_fields`
- What: Verifies incomplete JSON maps to `MissingFields`.
- Syntax: Inline raw string literal for JSON sample.
- Improvement/unusual: Good migration safety test.

### Test helper: `unique_test_path`
- What: Creates unique temp path based on nanoseconds.
- Syntax: `SystemTime::now().duration_since(...)`.
- Improvement/unusual:
  - Time-based uniqueness is usually enough; UUID would remove rare collision risk.

### Test helper: `write_test_file`
- What: Creates parent directories and writes fixture file content.
- Syntax: `if let Some(parent) = path.parent()`.
- Improvement/unusual: Straightforward.

---

## File: `src-tauri/src/integrations/daylite/mod.rs`

No functions in this file.  
It is the module index for `auth`, `client`, `contacts`, `projects`, `shared`.

---

## File: `src-tauri/src/integrations/daylite/shared.rs`

### `build_limit_query`
- What: Converts optional `limit` to query params.
- Syntax: mutable vector + `if let Some(limit)`.
- Improvement/unusual:
  - Could be a one-liner iterator style, but current explicit form is clearer.

### `normalize_base_url`
- What: Trims whitespace/trailing slash and rejects empty base URL.
- Syntax: `trim().trim_end_matches('/')`.
- Improvement/unusual:
  - Could validate URL format here; currently only emptiness is enforced.

### `load_daylite_tokens`
- What: Reads Daylite token fields from `LocalStore` into `DayliteTokenState`.
- Syntax: clones owned strings from store.
- Improvement/unusual: Clear and explicit.

### `store_daylite_tokens`
- What: Writes token state back into `LocalStore`.
- Syntax: mutable borrow `&mut LocalStore`.
- Improvement/unusual: Clear and symmetric with loader.

### `load_store_or_error`
- What: Delegates to local store loader and maps error type into Daylite API error type.
- Syntax: `map_err(map_store_error)`.
- Improvement/unusual: Good adapter boundary.

### `save_store_or_error`
- What: Same adapter pattern for store save path.
- Syntax: small wrapper function.
- Improvement/unusual: Good consistency.

### `normalize_http_error`
- What: Maps HTTP status code to user-facing error code/message plus technical details.
- Syntax:
  - if/else chain for status classes,
  - range check `(500..=599).contains(&status)`.
- Improvement/unusual:
  - Could use `match` on status/ranges to make mapping table-like.

### `missing_token_error`
- What: Convenience constructor for token-missing error shape.
- Syntax: returns struct literal with provided messages.
- Improvement/unusual: Fine utility function.

### `should_refresh_access_token`
- What: Decides if refresh is required now (missing token, missing expiry, or expiry within 10s).
- Syntax: `match` on optional expiry.
- Improvement/unusual:
  - 10-second buffer is hard-coded; constant would improve tunability/readability.

### `current_epoch_ms`
- What: Reads system time and converts to `u64` milliseconds with error mapping.
- Syntax:
  - `duration_since(UNIX_EPOCH)` may fail,
  - `u64::try_from(duration.as_millis())` checked conversion.
- Improvement/unusual: Defensive and correct.

### `truncate_for_log`
- What: Truncates long strings for safer technical logs.
- Syntax: char-count based truncation, then append `...`.
- Improvement/unusual:
  - Char iteration is Unicode-safe; good choice over byte slicing.

### `map_store_error`
- What: Converts local store error to Daylite error contract.
- Syntax: straight field mapping.
- Improvement/unusual:
  - Maps all store errors to `InvalidConfiguration`; this is simple but may lose granularity.

---

## File: `src-tauri/src/integrations/daylite/auth.rs`

### `daylite_connect_refresh_token`
- What:
  - Normalizes/validates base URL.
  - Exchanges refresh token via API client.
  - Persists base URL + token state in local store.
  - Returns boolean token-sync status.
- Syntax:
  - async command function returning `Result<DayliteTokenSyncStatus, DayliteApiError>`.
  - `?` chains fallible steps.
- Improvement/unusual:
  - Good command orchestration.
  - Could validate refresh token non-emptiness earlier for faster failure, though client already does it.

---

## File: `src-tauri/src/integrations/daylite/client.rs`

### `DayliteApiClient::new`
- What: Creates API client with reqwest transport for given base URL.
- Syntax: stores transport behind `Arc<dyn DayliteHttpTransport>`.
- Improvement/unusual:
  - Trait object is good for testability.

### `DayliteApiClient::with_transport` (`#[cfg(test)]`)
- What: Test-only constructor for injecting mock transport.
- Syntax: compiled only in tests via `#[cfg(test)]`.
- Improvement/unusual: Good test seam.

### `DayliteApiClient::exchange_refresh_token`
- What: Public wrapper that delegates to refresh flow.
- Syntax: small async forwarding function.
- Improvement/unusual:
  - Could be inlined by callers; kept for clearer intent/API.

### `DayliteApiClient::list_projects`
- What: Uses `/projects/_search` and unwraps search result envelope to plain list.
- Syntax: explicit generic `execute_json_request::<DayliteSearchResult<...>>()`.
- Improvement/unusual:
  - Assumes endpoint always uses search envelope; this is correct for chosen API contract.

### `DayliteApiClient::search_projects`
- What: Searches by project name with optional limit.
- Syntax: JSON body built with `json!` macro.
- Improvement/unusual:
  - Search filter is fixed to `name.contains`; maybe later configurable.

### `DayliteApiClient::list_contacts`
- What: Loads full contact records filtered by category `Monteur`.
- Syntax: POST search with `full-records=true`.
- Improvement/unusual:
  - Server-side filter + local filtering later is defensive but slightly redundant.

### `DayliteApiClient::search_contacts`
- What: Searches contacts by `full_name contains` with optional limit.
- Syntax: mut query vector with appended `full-records`.
- Improvement/unusual:
  - Could search nickname as well, depending on UX needs.

### `DayliteApiClient::update_contact_ical_urls`
- What:
  - Parses contact id from reference.
  - GETs current contact.
  - Merges iCal URLs with existing URLs.
  - PATCHes contact with merged URLs.
- Syntax:
  - multi-step async flow with token state propagation.
- Improvement/unusual:
  - Two-step read+write prevents clobbering unrelated URLs (good).
  - Potential race if another writer updates between GET and PATCH.

### `DayliteApiClient::execute_json_request`
- What:
  - Validates token presence.
  - Performs proactive refresh when needed.
  - Sends authenticated request.
  - Normalizes non-2xx HTTP errors.
  - Parses JSON to requested type.
- Syntax:
  - generic over `T: DeserializeOwned`,
  - Option/Result control flow with `?`.
- Improvement/unusual:
  - Does not retry on 401 if token considered valid; this is intentional but stricter than optimistic retry models.

### `DayliteApiClient::refresh_tokens`
- What: Calls refresh endpoint and validates token response fields and expiry.
- Syntax:
  - explicit checks for empty tokens and `expires_in == 0`,
  - uses `saturating_*` arithmetic for expiry timestamp.
- Improvement/unusual:
  - Strict snake_case parser is good for correctness; less tolerant to API shape drift.

### `DayliteApiClient::send_request`
- What: Converts args into request struct and delegates to transport.
- Syntax: small abstraction layer around transport trait call.
- Improvement/unusual: clean indirection point.

### Trait method: `DayliteHttpTransport::send`
- What: Interface for HTTP transport implementations.
- Syntax:
  - returns boxed future (`BoxFuture<'a, Result<...>>`) for async trait-object compatibility.
- Improvement/unusual:
  - This is the standard pre-`async fn in traits` style when trait objects are needed.

### `ReqwestTransport::new`
- What: Builds reqwest client and stores normalized base URL.
- Syntax: builder pattern with mapped errors.
- Improvement/unusual: solid.

### `ReqwestTransport::send` (trait impl)
- What: Composes URL/query, method, headers/body, executes request, returns status+body.
- Syntax:
  - `match` on custom method enum (`Get`, `Post`, `Patch`),
  - optional header/body setup with `if let`.
- Improvement/unusual:
  - URL built via `format!("{}{}", base_url, path)`; robust if `path` is always slash-prefixed (it is).

### `parse_refresh_response_body`
- What: Deserializes refresh JSON response into strict struct.
- Syntax: `serde_json::from_str::<DayliteRefreshTokenResponse>`.
- Improvement/unusual:
  - Good technical error with truncated body.

### `parse_contact_id`
- What: Extracts numeric id from reference like `/v1/contacts/100`.
- Syntax: `rsplit('/').next().unwrap_or_default().parse::<u64>()`.
- Improvement/unusual:
  - Unusual if references ever use non-numeric suffixes; currently this hard-fails (likely desired).

### `merge_contact_ical_urls`
- What: Keeps non-iCal URLs, replaces primary/absence iCal URLs with provided values.
- Syntax:
  - iterator filter + label normalization + conditional pushes.
- Improvement/unusual:
  - Label matching is heuristic (`contains` in lowercase). If label naming varies heavily, structured metadata would be safer.

### `is_primary_ical_label`
- What: Classifies labels for primary calendar URLs (`einsatz` or `termine`).
- Syntax: boolean `contains` checks.
- Improvement/unusual:
  - Heuristic and language-specific.

### `is_absence_ical_label`
- What: Classifies labels for absence calendar URLs (`abwesenheit` or `fehlzeiten`).
- Syntax: boolean `contains`.
- Improvement/unusual:
  - Same heuristic caveat as above.

### `normalize_label`
- What: Trims optional label and lowercases if non-empty.
- Syntax: Option chain (`map` -> `filter` -> `map`).
- Improvement/unusual: idiomatic and clear.

### `normalize_non_empty`
- What: Returns trimmed `&str` only if non-empty.
- Syntax: borrowed return with same input lifetime.
- Improvement/unusual:
  - Nice zero-allocation helper.

### Test: `list_projects_returns_typed_data_for_200_response`
- What: Verifies successful project list decode and request shape.
- Syntax: `tauri::async_runtime::block_on` around async assertions.
- Improvement/unusual: strong request assertion.

### Test: `list_projects_returns_normalized_401_error`
- What: Verifies 401 mapping.
- Syntax: `expect_err`.
- Improvement/unusual: good.

### Test: `list_projects_returns_normalized_429_error`
- What: Verifies 429 mapping.
- Syntax: negative-path assertion.
- Improvement/unusual: good.

### Test: `list_projects_returns_normalized_500_error`
- What: Verifies 5xx mapping.
- Syntax: negative-path assertion.
- Improvement/unusual: good.

### Test: `list_projects_returns_invalid_response_error_for_malformed_payload`
- What: Verifies non-envelope JSON causes `InvalidResponse`.
- Syntax: typed-deserialization failure path.
- Improvement/unusual: good.

### Test: `list_projects_refreshes_before_request_when_access_token_is_expired`
- What: Verifies proactive refresh when token expired.
- Syntax: mock response queue (refresh then list).
- Improvement/unusual: strong behavior contract.

### Test: `list_projects_refreshes_when_access_token_is_missing`
- What: Verifies refresh if access token is empty.
- Syntax: two-request expectation.
- Improvement/unusual: good.

### Test: `list_projects_uses_existing_access_token_when_it_is_still_valid`
- What: Verifies no refresh when token still valid.
- Syntax: single request assertion.
- Improvement/unusual: good.

### Test: `list_projects_fails_refresh_when_token_fields_are_not_snake_case`
- What: Verifies strict parser rejects camelCase token fields.
- Syntax: refresh failure assertion.
- Improvement/unusual:
  - This is intentionally strict but may break with API drift.

### Test: `list_projects_does_not_refresh_on_401_when_access_token_not_near_expiry`
- What: Verifies no automatic refresh retry on unauthorized response.
- Syntax: request count + endpoint assertions.
- Improvement/unusual:
  - Policy is strict; an alternative is one retry-on-401 with refresh.

### Test: `search_contacts_returns_typed_search_result`
- What: Verifies contact search decoding and query/body shape.
- Syntax: asserts query vector ordering and JSON body.
- Improvement/unusual: good.

### Test: `list_contacts_uses_full_records_search_endpoint`
- What: Verifies full-record contact listing call shape.
- Syntax: POST request + query/body assertions.
- Improvement/unusual: good.

### Test: `update_contact_ical_urls_patches_contact_urls_only`
- What: Verifies URL merge behavior and patch payload.
- Syntax: two sequential mock responses (GET then PATCH).
- Improvement/unusual: valuable regression test.

### Test: `returns_missing_token_error_when_no_tokens_are_available`
- What: Verifies explicit missing-token error behavior.
- Syntax: default token state + `expect_err`.
- Improvement/unusual: good.

### Test: `should_refresh_access_token_when_less_than_ten_seconds_remain`
- What: Verifies refresh threshold lower bound.
- Syntax: direct helper assertion.
- Improvement/unusual: good.

### Test: `should_not_refresh_access_token_when_more_than_ten_seconds_remain`
- What: Verifies refresh threshold upper bound.
- Syntax: direct helper assertion.
- Improvement/unusual: good.

### Test: `build_limit_query_returns_empty_query_without_limit`
- What: Verifies no query param without limit.
- Syntax: pure function assertion.
- Improvement/unusual: good.

### Test: `build_limit_query_sets_limit_parameter_when_present`
- What: Verifies correct limit param formatting.
- Syntax: pure function assertion.
- Improvement/unusual: good.

### Test helper: `MockTransport::new`
- What: Initializes queue-backed transport mock.
- Syntax: `Arc<Mutex<VecDeque<_>>>` shared mutable state in tests.
- Improvement/unusual: standard approach.

### Test helper: `MockTransport::requests`
- What: Returns cloned captured requests.
- Syntax: `lock().expect(...).clone()`.
- Improvement/unusual:
  - `expect` is acceptable in tests.

### Test trait impl: `MockTransport::send`
- What: Captures request and pops next mock response.
- Syntax: async boxed future in trait impl.
- Improvement/unusual:
  - Error on empty queue is useful fail-fast behavior.

### Test helper: `mock_response`
- What: Constructs mock HTTP response value.
- Syntax: small constructor helper.
- Improvement/unusual: straightforward.

---

## File: `src-tauri/src/integrations/daylite/projects.rs`

### `daylite_list_projects`
- What: Loads raw projects, maps them to normalized planning records, stores rotated token state.
- Syntax: iterator mapping `into_iter().map(...).collect()`.
- Improvement/unusual:
  - Good consolidation of mapping on backend.

### `daylite_search_projects`
- What: Forwards project search and returns raw search envelope.
- Syntax: async Tauri command wrapper around client method.
- Improvement/unusual:
  - Return type differs from list command (raw summary vs normalized record). This asymmetry is intentional but worth noting.

### `map_daylite_project_summary`
- What: Converts raw API summary into normalized planning record.
- Syntax: struct literal with helper calls per field.
- Improvement/unusual: clear mapping layer.

### `map_project_status`
- What: Maps optional status string to strongly typed `PlanningProjectStatus`, defaulting to `NewStatus`.
- Syntax: lowercase normalization + if-chain.
- Improvement/unusual:
  - Could be a `match normalized.as_str()` for readability.
  - Silent fallback is safe but can hide new upstream status values.

### `normalize_reference`
- What: Trims reference string.
- Syntax: `trim().to_string()`.
- Improvement/unusual: simple helper.

### `normalize_optional_string`
- What: Trims optional string and drops empty values.
- Syntax: `and_then` transformation.
- Improvement/unusual: idiomatic.

### `normalize_keywords`
- What: Trims keyword entries and removes empties.
- Syntax: `filter_map` over vector.
- Improvement/unusual: good cleanup.

### `normalize_optional_date`
- What: Normalizes date/time strings to UTC RFC3339 millis; accepts full RFC3339 or `YYYY-MM-DD`.
- Syntax:
  - `DateTime::parse_from_rfc3339`
  - fallback `NaiveDate::parse_from_str`
  - optional chaining with `?`.
- Improvement/unusual:
  - Invalid input drops to `None` silently; logging/metric could help diagnose bad upstream data.

### Test: `maps_project_summary_to_planning_project_record`
- What: Verifies end-to-end project mapping, status normalization, keyword cleanup, date normalization.
- Syntax: table-style assertion with explicit expected values.
- Improvement/unusual: strong coverage.

### Test: `defaults_unknown_project_status_to_new_status`
- What: Verifies unknown statuses map to fallback enum variant.
- Syntax: direct helper assertion.
- Improvement/unusual: good explicit policy test.

---

## File: `src-tauri/src/integrations/daylite/contacts.rs`

### `daylite_list_contacts`
- What:
  - Loads contacts from Daylite.
  - Maps to normalized planning records.
  - Filters `Monteur` contacts.
  - Sorts by display name.
  - Persists token rotation + contact cache + sync timestamp.
- Syntax:
  - chained iterator pipeline and owned-to-owned mapping.
- Improvement/unusual:
  - Applies category filter both upstream and locally; redundant but safer.

### `daylite_search_contacts`
- What: Search command pass-through; updates stored token state.
- Syntax: async command wrapper.
- Improvement/unusual:
  - Returns raw `DayliteContactSummary` envelope, while list returns normalized records.

### `daylite_update_contact_ical_urls`
- What:
  - Updates iCal URLs via client.
  - Maps updated record.
  - Mutates cached contacts in-memory (replace by reference, keep only Monteur).
  - Saves updated cache and timestamp.
- Syntax:
  - `retain` for remove-by-predicate.
  - conditionally `push`.
- Improvement/unusual:
  - Full contact cache clone is simple but not most efficient; in-place index update could avoid cloning.

### `daylite_list_cached_contacts`
- What: Reads cached contacts from local store and applies same filter/sort as live list.
- Syntax: sync command (non-async) because only local I/O.
- Improvement/unusual:
  - Good for offline/startup performance.

### `map_daylite_contact_summary`
- What: Maps raw Daylite contact to normalized planning record with full-name fallback.
- Syntax: `or_else` fallback chain for optional full name.
- Improvement/unusual: clear and robust.

### `map_cached_contact`
- What: Maps cached entry back to runtime planning record.
- Syntax: fallback to cached `display_name` when `full_name` missing.
- Improvement/unusual:
  - Good backward compatibility with older cache shape.

### `map_planning_contact_to_cache_entry`
- What: Converts normalized runtime record into cache entry format.
- Syntax: nested `into_iter().map(...).collect()` for URL entries.
- Improvement/unusual: straightforward.

### `filter_monteur_contacts`
- What: Retains only contacts recognized as Monteur.
- Syntax: `into_iter().filter(...).collect::<Vec<_>>()`.
- Improvement/unusual: clear.

### `is_monteur_contact`
- What: Category predicate (`"monteur"` case-insensitive).
- Syntax: option chain + lowercase comparison.
- Improvement/unusual:
  - Locale-agnostic lowercase is fine here but note language-specific heuristic.

### `sort_contacts`
- What: Sorts contacts by case-insensitive display name.
- Syntax: `sort_by` with computed comparison keys.
- Improvement/unusual:
  - Recomputes display names during compares; could precompute keys for large lists.

### `contact_display_name`
- What: Chooses nickname first, then full name, else fallback label.
- Syntax: early returns from `if let Some(...)`.
- Improvement/unusual: clear UX rule.

### `normalize_contact_urls`
- What: Trims URL fields and drops entries with no useful data.
- Syntax: `filter_map` + constructed normalized struct.
- Improvement/unusual: useful data hygiene.

### `normalize_cached_contact_urls`
- What: Same normalization logic for cached URL type.
- Syntax: same functional pattern as above.
- Improvement/unusual:
  - Duplicate logic vs previous function; could be shared with a generic helper.

### `normalize_string`
- What: Trims required string field.
- Syntax: simple helper.
- Improvement/unusual: okay.

### `normalize_string_option`
- What: Trims optional strings and converts empty to `None`.
- Syntax: `and_then` cleanup.
- Improvement/unusual: idiomatic.

### `normalize_optional_string`
- What: Alias wrapper around `normalize_string_option`.
- Syntax: forwarding helper.
- Improvement/unusual:
  - Slightly redundant name alias; may be kept for semantic readability.

### `join_name`
- What: Builds full name from first and last names, skipping empties.
- Syntax:
  - array literal + iterator + `collect::<Vec<_>>().join(" ")`.
- Improvement/unusual:
  - Could avoid intermediate `Vec` with manual push + `format!`, but current style is readable.

### `current_timestamp_iso8601`
- What: Emits current UTC timestamp in RFC3339 millis format.
- Syntax: `Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)`.
- Improvement/unusual: good canonical timestamp.

### Test: `maps_daylite_contact_summary_to_planning_contact_record`
- What: Validates trimming, fallback name composition, category cleanup, URL normalization.
- Syntax: rich fixture + exact expected struct assertion.
- Improvement/unusual: strong mapping test.

### Test: `maps_cached_contact_with_display_name_fallback`
- What: Validates cache-to-runtime mapping fallback behavior.
- Syntax: fixture-based unit test.
- Improvement/unusual: good migration safety.

### Test: `filters_and_sorts_monteur_contacts_by_display_name`
- What: Validates category filtering and deterministic sort order.
- Syntax: asserts length and reference ordering.
- Improvement/unusual: good.

---

## File: `src-tauri/Cargo.lock`

No Rust functions here.  
Only dependency resolution updates (version/checksum graph).

---

## Cross-file observations (potential improvements)

1. Return-shape asymmetry:
   - list endpoints return normalized records, search endpoints return raw summaries.
   - This may be intentional, but it adds two frontend data contracts.
2. Status and category heuristics:
   - `map_project_status` and iCal label/category checks are string-based heuristics.
   - Consider central constants and telemetry for unknown values.
3. Error granularity:
   - `map_store_error` collapses multiple local-store failures into `InvalidConfiguration`.
   - If debugging production issues, preserving the original category could help.
4. Duplication:
   - URL normalization exists in two contact helper functions with very similar logic.
5. Retry strategy:
   - current client does proactive refresh but no retry-on-401 when token was expected valid.
   - An optional single retry could improve resilience in some edge cases.
