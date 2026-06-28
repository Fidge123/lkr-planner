## 1. Setup and Types

- [ ] 1.1 Add reqwest and serde dependencies to Cargo.toml
- [ ] 1.2 Create Planradar client module structure
- [ ] 1.3 Define typed models for Project, ProjectStatus, CreateProjectRequest

## 2. Client Implementation

- [ ] 2.1 Implement PlanradarClient struct with HTTP client
- [ ] 2.2 Add static API token authentication header handling
- [ ] 2.3 Implement project search/list method (paginated: sort, page, pagesize)
- [ ] 2.4 Implement blank project create method (POST projects)
- [ ] 2.5 Implement copy-project method with name and aspect toggles (details, groups, ticket_types, users, components)
- [ ] 2.6 Implement project status read method
- [ ] 2.7 Implement project reactivate method via archive_project with status 1

## 3. Error Handling

- [ ] 3.1 Define PlanradarError enum
- [ ] 3.2 Implement error mapping from API responses
- [ ] 3.3 Add retry logic for transient failures

## 4. Configuration

- [ ] 4.1 Add the Customer ID and tenant/account configuration options in the local config store
- [ ] 4.2 Store the user-provided API token in the OS keychain via the secret manager (service `lkr-planner-planradar`, username `LKR Planner Planradar Token`)
- [ ] 4.3 Include the configured Customer ID as the `/api/v1/{customer_id}/...` path segment in all requests

## 5. Testing

- [ ] 5.1 Write unit tests for success and API error mappings
- [ ] 5.2 Write tests for auth and rate-limit behavior