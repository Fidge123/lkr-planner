# ADR 0005: Local Configuration and Cache Store

- Status: Accepted
- Date: 2026-02-13

## Context

BL-005 requires restart-safe persistence for local configuration values and an optional local cache for recently loaded Daylite data.
The app already routes frontend state changes through Tauri commands, and no additional storage dependency should be introduced for this step.

## Decision

- Implement a file-backed local store in Rust under the Tauri app config directory (`app_config_dir`).
- Persist one typed JSON payload (`local-store.json`) that contains:
  - API endpoints
  - token references
  - employee-specific settings
  - project proposal filters
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

## Consequences

- Configuration survives application restarts without introducing new dependencies.
- File corruption and partial edits are reported in a user-friendly way while retaining technical diagnostics.
- Future frontend services can consume the same typed command contract for all local settings and cache interactions.
