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
- Planradar uses a static API token rather than OAuth
- No refresh or rotation flow is required (unlike Daylite, ADR-0006)
- The token is user-provided and stored in the OS keychain via the existing secret manager (`secret_manager.rs`, archived BL-040), never in the local config store
- The client attaches the token to each request and surfaces auth failures via the normalized error type

### Tenant selection
**Decision**: The app operates against exactly one correct tenant, but the user may belong to several
- Open question: does the API token already scope requests to a single tenant, or must the user pick the tenant from a list?
- This must be investigated against the Planradar API before the client finalizes how the tenant is resolved (see open points)
- If selection is required, the chosen tenant is persisted in the local config store (non-secret)

### Error Handling
**Decision**: Normalize all API errors into standardized error types
- Map Planradar error responses to internal error enum
- Include status code and error message for debugging

### Configuration
**Decision**: Split storage by sensitivity
- The Planradar base URL already exists in the local store (`planradar_base_url`)
- Non-secret tenant/account settings are stored in the local config store alongside it
- The API token is stored only in the OS keychain via the secret manager
- Allows switching between environments without code changes

## Risks / Trade-offs

- **Risk**: Planradar API rate limiting
  - **Mitigation**: Implement exponential backoff retry logic

- **Risk**: API changes breaking the client
  - **Mitigation**: Version the client and document API contract