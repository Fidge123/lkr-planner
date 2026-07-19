## Context

The application needs to integrate with Planradar for project management. The Planradar API provides project CRUD operations. This change implements the foundational API client that other Planradar features will depend on.

## Goals / Non-Goals

**Goals:**
- Provide typed Planradar client for project operations
- Normalize error responses for consistent frontend error handling
- Support configurable tenant/account settings

**Non-Goals:**
- UI implementation for project management
- Advanced project filtering or sorting
- Bulk operations

## Decisions

### Client Architecture
**Decision**: Use reqwest for HTTP client with typed response models
- Planradar API returns JSON responses
- Typed models provide compile-time safety
- Mirror the existing Daylite client structure (transport trait plus VCR replay cassettes per ADR-0010)

### Project creation mechanism
**Decision**: Use the dedicated copy-project endpoint for source-based creation, and the create-project endpoint for blank creation
- Planradar exposes `POST /api/v1/{customer_id}/projects/{project_id}/copy_project` with a new `name` and boolean toggles `details`, `groups`, `ticket_types` (forms), `users`, `components` (layers)
- This is the same "copy project" affordance offered in the Planradar UI, so it is preferred over a manual read-then-recreate
- Blank creation uses `POST /api/v1/{customer_id}/projects` with `data.attributes` (name, street, zipcode, city, country, description, start/end dates)
- The copy is server-side; field-level edits happen afterward via `PUT /api/v1/{customer_id}/projects/{project_id}` (see BL-037 hybrid flow)

### Project reactivation mechanism
**Decision**: Reactivate via the archive-project endpoint, not a separate reopen endpoint
- Planradar has no dedicated reactivate endpoint; archive and unarchive share `PUT /api/v1/{customer_id}/projects/{project_id}/archive_project`
- The body sets `data.attributes.status`: `9` archives, `1` unarchives
- Reactivation therefore sends status `1`

### Authentication
**Decision**: Authenticate with a static, user-provided API token, separate from Daylite auth
- Planradar uses a static personal access token sent in the `X-PlanRadar-API-Key` header, not OAuth
- No refresh or rotation flow is required (unlike Daylite, ADR-0006)
- The token is user-provided and stored in the OS keychain via the existing secret manager (`secret_manager.rs`, archived BL-040) under service `lkr-planner-planradar` and username `LKR Planner Planradar Token`, matching the Daylite convention (`lkr-planner-daylite` / `LKR Planner Daylite Token`); never in the local config store
- The client attaches the token to each request and surfaces auth failures via the normalized error type

### Tenant selection
**Decision**: The customer/account is a configured Customer ID supplied per request, not derived from the token
- Verified against the Planradar Open API: endpoints are scoped by a path segment, e.g. `GET /api/v1/{customer_id}/projects`
- The personal access token is user-based and may grant access to several customers, so it does not by itself select the tenant
- The user provides the single correct Customer ID (Account ID, from PlanRadar Settings > Account), mirroring PlanRadar Connect configuration (API Key + Customer ID + URL)
- The Customer ID is non-secret and stored in the local config store alongside `planradar_base_url`
- No tenant picker list is required; manual entry of the one correct Customer ID is sufficient

### Error Handling
**Decision**: Normalize all API errors into standardized error types
- Map Planradar error responses to internal error enum
- Include status code and error message for debugging

### Configuration
**Decision**: Split storage by sensitivity
- The Planradar base URL already exists in the local store (`planradar_base_url`)
- The non-secret Customer ID (and any other tenant/account settings) is stored in the local config store alongside it
- The API token is stored only in the OS keychain via the secret manager
- Allows switching between environments without code changes

### Rate limiting
**Decision**: Enforce a conservative client-side request budget in addition to retry backoff
- Planradar allows roughly 30 requests per minute per token and imposes a long forced cooldown (during which all requests are rejected) once that is exceeded, so reactive backoff alone is not enough
- A process-wide sliding-window limiter caps outbound requests at 15 per 60 seconds, shared across all commands (each builds its own client) so the budget is global per account token
- The cap is set well below 30 because the same personal token may be used by other tools or sessions at the same time
- Retries count against the budget because each retry is a real request
- Only idempotent requests (GET, and the idempotent PUT archive_project) are auto-retried; POST create/copy are never retried, to avoid duplicate projects when a response is lost
- If Planradar itself returns a 429, the limiter enters a cooldown (at least the rate window) during which all requests are held back, and that 429 is surfaced immediately rather than retried, so the client stops feeding a forced cooldown that would otherwise be prolonged
- Planradar does not send a `Retry-After` header, so the cooldown is a fixed conservative duration rather than server-driven
- A single request waits at most a bounded time for the budget before returning a rate-limit error, so a burst of commands cannot hang a Tauri call indefinitely

## Risks / Trade-offs

- **Risk**: Planradar API rate limiting and forced cooldown on overuse
  - **Mitigation**: Conservative client-side sliding-window limiter (15 req/60s), a cooldown that engages when Planradar returns a 429, and exponential backoff retry on transient (5xx/network) responses

- **Risk**: API changes breaking the client
  - **Mitigation**: Version the client and document API contract