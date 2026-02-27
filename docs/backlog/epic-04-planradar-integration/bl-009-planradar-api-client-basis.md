# BL-009: Planradar API Client (Basis)

## Scope
- Provide minimal typed Planradar client for:
  - project search/list
  - project create (template-based when required)
  - project status read (active/archived/reopen support)
- Normalize API error payloads for frontend usage.
- Keep tenant/account settings configurable.

## Acceptance Criteria
- All client responses are typed.
- Error payloads are standardized and consumable by UI/services.
- Tenant/account configuration can be switched without code changes.

## Tests (write first)
- Unit tests for success and API error mappings.
- Auth and rate-limit behavior tests.
