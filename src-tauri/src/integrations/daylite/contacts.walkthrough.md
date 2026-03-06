# Walkthrough: `src-tauri/src/integrations/daylite/contacts.rs`

## Purpose

This file handles Daylite contact retrieval, cached contact reads, and updating the iCal URLs stored on a contact. It also contains the normalization rules that turn raw Daylite responses into stable planning records.

## Block by block

### Imports (`lines 1-14`)

- The file pulls in cache entry types from `local_store`, auth and client helpers, shared token/store helpers, `chrono` for timestamps, and `serde`/`specta` for API contracts.

### API and frontend data types (`lines 16-66`)

- `DayliteContactSummary` models the Daylite API payload. The `alias` attributes accept both `first_name`/`last_name` style fields and the Rust camelCase field names.
- `DayliteContactUrl` models one contact URL record.
- `DayliteUpdateContactIcalUrlsInput` is the frontend command payload for the mutation endpoint.
- `PlanningContactRecord` is the normalized shape used by the rest of the app.

Rust syntax to notice:
- `#[serde(rename = "self")]` maps the JSON field named `self` onto the Rust field `reference`.
- `skip_serializing_if` avoids emitting empty optional fields back to JSON.

Best practice:
- Separate raw API models from normalized app models when you need cleanup logic in between.

### Public list command (`lines 68-87`)

- `daylite_list_contacts` loads the local store, builds a client, calls `list_contacts_core`, then persists both refreshed token state and the latest contact cache.
- `last_synced_at` is updated with the current UTC timestamp.

Best practice:
- Cache writes happen only after a successful API call, which avoids storing partial state.

### Public update command (`lines 89-107`)

- `daylite_update_contact_ical_urls` follows the same pattern: load store, build client, call the core mutation helper, persist refreshed tokens and cache metadata, then return the updated contact.

### Core mutation flow (`lines 109-163`)

- `update_contact_ical_urls_core` extracts the numeric contact id from the Daylite reference.
- It fetches the current contact with `GET /contacts/{id}`.
- It merges the existing URLs with the new primary and absence iCal URLs, removing only the labels this app manages.
- It sends a `PATCH /contacts/{id}` with the merged URL array.
- It normalizes the response, updates the in-memory cache, and removes cached entries that no longer belong in the Monteur list.

Rust syntax to notice:
- Passing `store: &mut LocalStore` lets the function update cache state in place.
- `retain` mutates a vector by keeping only elements that match the predicate.

Best practice:
- Read-modify-write is the right pattern when the server object contains fields your app does not fully own.

### Core read flow (`lines 165-192`)

- `list_contacts_core` sends a search request to `/contacts/_search` with a category filter for `"Monteur"`.
- The results are normalized, filtered again locally, and sorted by display name.

Note:
- The local filter duplicates the API filter intentionally so cached or replayed data still passes through the same business rules.

### Cached contact command (`lines 194-208`)

- `daylite_list_cached_contacts` reads the store only and returns normalized cached contacts after applying the same filter-and-sort pipeline used for live results.

Best practice:
- Reuse the same normalization pipeline for live and cached data to avoid subtle UI differences.

### Mapping functions (`lines 210-249`)

- `map_daylite_contact_summary` builds a planning record from the API response and falls back to `join_name` when `full_name` is missing.
- `map_cached_contact` turns cached JSON back into the same planning record type.
- `map_planning_contact_to_cache_entry` converts normalized runtime data back into the persisted cache shape.

Rust syntax to notice:
- `or_else(...)` lazily computes a fallback only when the first option is `None`.
- The contact cache conversion uses iterator chains with `.map(...).collect()`.

### Filtering and sorting helpers (`lines 251-283`)

- `filter_monteur_contacts` and `is_monteur_contact` enforce the category rule.
- `sort_contacts` orders contacts case-insensitively by `contact_display_name`.
- `contact_display_name` prefers nickname, then full name, then the fallback `"Unbenannter Kontakt"`.

Best practice:
- Put display-specific sorting logic in a named helper so the intent is obvious and reusable.

### String and URL normalization helpers (`lines 285-365`)

- `normalize_contact_urls` and `normalize_cached_contact_urls` trim fields and drop empty URL entries.
- `normalize_string`, `normalize_string_option`, and `normalize_optional_string` centralize trimming behavior.
- `join_name` combines first and last name safely.
- `current_timestamp_iso8601` returns a consistent UTC timestamp format with millisecond precision.

Rust syntax to notice:
- `filter_map` is a good fit when each input item may either produce one output or be dropped.
- `collect::<Vec<_>>()` asks the compiler to gather iterator output into a vector.

### Contact id parsing and iCal URL merge rules (`lines 367-438`)

- `parse_contact_id` takes the last path segment from a Daylite reference such as `/v1/contacts/500` and parses it as `u64`.
- `merge_contact_ical_urls` preserves unrelated URLs, removes only existing managed iCal labels, and then appends replacement entries for the provided non-empty URLs.
- `is_primary_ical_label`, `is_absence_ical_label`, `normalize_url_label`, and `normalize_non_empty` support that merge logic.

Best practice:
- The merge deliberately targets labels, not array positions, which is much safer against server-side ordering changes.

### Tests (`lines 440-876`)

- The tests cover normalization, filtering, sorting, URL merge behavior, invalid references, live update flow, cache updates, cache removals, and VCR replay.
- `MockTransport` mirrors the pattern used in other Daylite modules: queued responses plus recorded outgoing requests.
- The mutation tests assert both behavior and outbound request shape, especially the generated PATCH body.

Rust syntax to notice:
- `VecDeque` is useful when tests need FIFO response queues.
- Async tests are run via `tauri::async_runtime::block_on(...)`.

## Best practices this file demonstrates

- Normalize third-party data before it reaches the rest of the application.
- Preserve server-owned fields when applying partial mutations.
- Keep live and cached code paths behaviorally aligned.
