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
- [x] 6.3 (red) Test retry with backoff on transient responses, and that rate-limit responses engage a cooldown instead of being retried
- [x] 6.4 (green) Implement retry logic

## 7. Configuration

- [x] 7.1 Add the Customer ID and tenant/account options to the local config store
- [x] 7.2 Store the user-provided API token in the OS keychain via the secret manager (service `lkr-planner-planradar`, username `LKR Planner Planradar Token`)

## 8. Follow-ups from code review

- [ ] 8.1 Establish which project ID form the path endpoints accept.
      The recorded cassettes show a numeric ID in the request path (`/projects/1569651`) but a hashed ID in the response (`data.id` is `ymmpayd`), and the list endpoint returns the hashed form too.
      So `PlanradarProject.id` may not be usable as input to `planradar_get_project_status` or `planradar_reactivate_project`, and `planradar_copy_project` now returns an ID it read from the project list, which inherits the same question.
      Verify against the live API whether the hashed form is accepted in paths.
      If it is, record a cassette that reads a project by its hashed ID so replay locks the round trip in.
      If it is not, carry the caller-supplied ID on `PlanradarProject` instead of the returned one, and document which form the field holds.
- [ ] 8.2 Validate the Customer ID as a path segment on the connect path.
      `projects_path` interpolates `customer_id` without calling `validate_path_segment`, and `planradar_connect` only trims it before probing, so a value such as `../../9999` escapes the customer scope during the probe and is then persisted into a config that `load_config` rejects on every later call.
      Make `projects_path` return `Result` and validate, mirroring `project_path`.
- [ ] 8.3 Stop returning raw Planradar response bodies to the frontend.
      `missing_field_error` and `normalize_http_error` embed up to 400 characters of the response body in `technical_message`, which is serialized to the frontend, and project payloads carry an `access-token` attribute plus customer addresses.
      Log the payload instead, or restrict the diagnostic to a whitelist of keys.
