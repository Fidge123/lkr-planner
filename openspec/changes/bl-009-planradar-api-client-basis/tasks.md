## 1. Setup and Types

- [ ] 1.1 Add reqwest and serde dependencies to Cargo.toml
- [ ] 1.2 Create Planradar client module structure
- [ ] 1.3 Define typed models for Project, ProjectStatus, CreateProjectRequest

## 2. Client Implementation

- [ ] 2.1 Implement PlanradarClient struct with HTTP client
- [ ] 2.2 Add authentication header handling
- [ ] 2.3 Implement project search/list method
- [ ] 2.4 Implement project create method
- [ ] 2.5 Implement project status read method
- [ ] 2.6 Implement project reactivate method

## 3. Error Handling

- [ ] 3.1 Define PlanradarError enum
- [ ] 3.2 Implement error mapping from API responses
- [ ] 3.3 Add retry logic for transient failures

## 4. Configuration

- [ ] 4.1 Add tenant/account configuration options
- [ ] 4.2 Support environment variable configuration

## 5. Testing

- [ ] 5.1 Write unit tests for success and API error mappings
- [ ] 5.2 Write tests for auth and rate-limit behavior