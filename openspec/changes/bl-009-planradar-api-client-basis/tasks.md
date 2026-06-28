## 1. Setup and Types

- [x] 1.1 Add reqwest and serde dependencies to Cargo.toml
- [x] 1.2 Create Planradar client module structure (mirror the Daylite transport trait and VCR harness, ADR-0010)
- [x] 1.3 Define typed models for Project, ProjectStatus, CreateProjectRequest, CopyProjectOptions

## 2. Request construction and auth (TDD)

- [x] 2.1 (red) Test that requests attach the `X-PlanRadar-API-Key` header and build the `/api/v1/{customer_id}/...` path
- [x] 2.2 (green) Implement PlanradarClient with HTTP client, static token auth, and Customer ID path construction

## 3. Project read and list (TDD)

- [x] 3.1 (red) Cassette test for project status read mapping (active vs archived)
- [x] 3.2 (green) Implement project status read method
- [x] 3.3 (red) Cassette test for paginated list (sort, page, pagesize)
- [x] 3.4 (green) Implement project search/list method

## 4. Project create and copy (TDD)

- [x] 4.1 (red) Test copy-project maps name and toggles (details, groups, ticket_types, users, components) to query params
- [x] 4.2 (green) Implement copy-project method
- [x] 4.3 (red) Cassette test for blank create returning the new project ID
- [x] 4.4 (green) Implement blank project create method (POST projects)

## 5. Reactivation (TDD)

- [x] 5.1 (red) Test reactivate sends archive_project with `data.attributes.status` set to 1
- [x] 5.2 (green) Implement project reactivate method

## 6. Error handling (TDD)

- [x] 6.1 (red) Tests mapping API error payloads (auth failure, rate limit, not found) to PlanradarError
- [x] 6.2 (green) Define PlanradarError enum and implement error mapping
- [x] 6.3 (red) Test retry with backoff on transient and rate-limit responses
- [x] 6.4 (green) Implement retry logic

## 7. Configuration

- [x] 7.1 Add the Customer ID and tenant/account options to the local config store
- [x] 7.2 Store the user-provided API token in the OS keychain via the secret manager (service `lkr-planner-planradar`, username `LKR Planner Planradar Token`)
