# ADR 0005: Local Configuration and Cache Store

- Status: Accepted
- Date: 2026-02-13
- Amended: 2026-10-23 (the store is held in memory and changed through atomic updates)

## Context

BL-005 requires restart-safe persistence for local configuration values and an optional local cache for recently loaded Daylite data.
The app already routes frontend state changes through Tauri commands, and no additional storage dependency should be introduced for this step.

## Decision

- Implement a file-backed local store in Rust under the Tauri app config directory (`app_config_dir`).
- Persist one typed JSON payload (`local-store.json`) that contains:
  - API endpoints
  - token references
  - employee-specific settings
  - Standard-Filter (pipelines, columns, categories, exclusion status)
  - contact filter (default keyword `Monteur`)
  - OpenRouteService routing settings (API key, profile)
  - optional Daylite cache data (recent projects/contacts + last sync timestamp)
- Use strict deserialization for persisted files:
  - missing required fields are treated as invalid (`MissingFields`)
  - malformed JSON is treated as corrupted (`CorruptFile`)
- Return structured error payloads with:
  - German user message (`userMessage`)
  - technical debug detail (`technicalMessage`)
  - machine-readable code (`code`)
- If no store file exists, load defaults instead of failing.

## Amendment 2026-10-23

Every command read and parsed `local-store.json` again, and every writer saved a whole copy it had loaded earlier.
A writer that awaited the network in between, such as the holiday import or the calendar test, overwrote whatever another command saved meanwhile.

- The file is read once per process, and later reads come from memory.
- Writers change the store only through `update_local_store`, which applies the change to the latest copy, writes the file and only then replaces the copy in memory.
- Writers that need the network compute their result first and apply it in one update afterwards.
- Edits made to the file while the app runs are not picked up until the next start.

## Consequences

- Configuration survives application restarts without introducing new dependencies.
- File corruption and partial edits are reported in a user-friendly way while retaining technical diagnostics.
- Future frontend services can consume the same typed command contract for all local settings and cache interactions.
