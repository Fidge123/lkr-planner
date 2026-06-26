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
**Decision**: Implement creation as read-then-create, not native clone
- Planradar exposes no clone or template endpoint
- To create from a source project, read that project's data and POST it as the body of a new create request
- This matches the "create from existing project" affordance in the Planradar UI

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

## Risks / Trade-offs

- **Risk**: Planradar API rate limiting
  - **Mitigation**: Implement exponential backoff retry logic

- **Risk**: API changes breaking the client
  - **Mitigation**: Version the client and document API contract