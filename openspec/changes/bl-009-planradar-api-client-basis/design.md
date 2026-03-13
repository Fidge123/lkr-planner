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

### Error Handling
**Decision**: Normalize all API errors into standardized error types
- Map Planradar error responses to internal error enum
- Include status code and error message for debugging

### Configuration
**Decision**: Use environment variables or config file for tenant/account settings
- Allows switching between environments without code changes

## Risks / Trade-offs

- **Risk**: Planradar API rate limiting
  - **Mitigation**: Implement exponential backoff retry logic

- **Risk**: API changes breaking the client
  - **Mitigation**: Version the client and document API contract