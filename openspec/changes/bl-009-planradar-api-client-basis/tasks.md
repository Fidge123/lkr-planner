## 1. Setup and Types

- [ ] 1.1 Add reqwest and serde dependencies to Cargo.toml
- [ ] 1.2 Create Planradar client module structure (mirror the Daylite transport trait and VCR harness, ADR-0010)
- [ ] 1.3 Define typed models for Project, ProjectStatus, CreateProjectRequest, CopyProjectOptions

## 2. Request construction and auth (TDD)

- [ ] 2.1 (red) Test that requests attach the `X-PlanRadar-API-Key` header and build the `/api/v1/{customer_id}/...` path
- [ ] 2.2 (green) Implement PlanradarClient with HTTP client, static token auth, and Customer ID path construction

## 3. Project read and list (TDD)

- [ ] 3.1 (red) Cassette test for project status read mapping (active vs archived)
- [ ] 3.2 (green) Implement project status read method
- [ ] 3.3 (red) Cassette test for paginated list (sort, page, pagesize)
- [ ] 3.4 (green) Implement project search/list method

## 4. Project create and copy (TDD)

- [ ] 4.1 (red) Test copy-project maps name and toggles (details, groups, ticket_types, users, components) to query params
- [ ] 4.2 (green) Implement copy-project method
- [ ] 4.3 (red) Cassette test for blank create returning the new project ID
- [ ] 4.4 (green) Implement blank project create method (POST projects)

## 5. Reactivation (TDD)

- [ ] 5.1 (red) Test reactivate sends archive_project with `data.attributes.status` set to 1
- [ ] 5.2 (green) Implement project reactivate method

## 6. Error handling (TDD)

- [ ] 6.1 (red) Tests mapping API error payloads (auth failure, rate limit, not found) to PlanradarError
- [ ] 6.2 (green) Define PlanradarError enum and implement error mapping
- [ ] 6.3 (red) Test retry with backoff on transient and rate-limit responses
- [ ] 6.4 (green) Implement retry logic

## 7. Configuration

- [ ] 7.1 Add the Customer ID and tenant/account options to the local config store
- [ ] 7.2 Store the user-provided API token in the OS keychain via the secret manager (service `lkr-planner-planradar`, username `LKR Planner Planradar Token`)
