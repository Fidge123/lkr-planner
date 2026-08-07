# Planradar Open API Findings

Summary of the Planradar Open API facts that drive the BL-009, BL-010, BL-037 and BL-038 designs.
Derived from the official Swagger 2.0 document (`PlanRadar's API Documentation`, version 2.0).
This is a curated summary, not the full endpoint list.

## Authentication

Requests authenticate with a static personal access token.
The token is sent in the `X-PlanRadar-API-Key` request header.
There is no OAuth flow and no token rotation.
Tokens are user-based and require an in-house user with the `API Access` permission (Pro and Enterprise accounts only).

## Tenant scoping

Every functional endpoint is nested under a customer segment, for example `GET /api/v1/{customer_id}/projects`.
The personal access token is user-based and may grant access to several customers, so it does not by itself select the tenant.
The API exposes no endpoint that lists the customers or accounts a token can reach.
The only path without a `customer_id` segment is `GET /extend_session`.
As a result a tenant dropdown cannot be populated from the API.
The user must supply the single correct Customer ID (Account ID, from Planradar Settings > Account), which is stored as non-secret local config.

## Rate limiting

The default limit is 30 requests per minute.
The client should apply retry with backoff for transient failures and rate-limit responses.

## Project creation

Blank creation uses `POST /api/v1/{customer_id}/projects` with a `data.attributes` body.
Known attribute keys are `name`, `street`, `zipcode`, `city`, `country`, `description`, and the start/end dates.
The Swagger documents the start and end dates under the hyphenated keys `drstart-date` and `drend-date` (the variants that carry descriptions and examples); the client sends those exact keys.
Unknown attribute keys are silently ignored by the API (a 2xx is still returned), so a wrong key drops the value without any error, which is why the exact key spelling must be verified against a recorded create cassette.

## Cassette recording and replay

Cassette matching is exact on method, path, query, and body, so a replay test only matches a recorded interaction if it sends a byte-identical request.
Both the recording harness and the replay tests build their requests from the shared `vcr_fixtures` helpers in `planradar/projects.rs`, so the request shapes cannot drift apart.
Those helpers read the same `PLANRADAR_CUSTOMER_ID`, `PLANRADAR_VCR_PROJECT_ID`, and `PLANRADAR_VCR_NEW_PROJECT_NAME` variables the harness records with, falling back to fixed literals.
This means the replay tests must run with the same environment that was used to record the cassettes; with those variables unset they only match cassettes recorded against the fallback values.

Copying a source project uses `POST /api/v1/{customer_id}/projects/{project_id}/copy_project`.
This is the same copy feature offered in the Planradar UI.
It takes a new `name` plus boolean toggles that select which aspects to copy:
details, groups, ticket_types (forms), users, and components (layers).
The copy is performed server-side, so field-level edits happen afterward via project update.

Copying is asynchronous, confirmed by a live recording.
The endpoint answers `202 Accepted` with a body of `{"job_id": "<uuid>"}` and no project payload, so the new project ID is not available when the call returns.
The Swagger documents only the 404 and 406 responses for this endpoint, so this success shape cannot be derived from the spec.
The client therefore returns the job ID; resolving it to the created project is left to the consuming feature.

## Project read and list

List uses `GET /api/v1/{customer_id}/projects` with pagination via `page`, `pagesize` and `sort`.
A single project is read via `GET /api/v1/{customer_id}/projects/{project_id}`.
A project is updated via `PUT /api/v1/{customer_id}/projects/{project_id}`.

## Archive and reactivate

There is no dedicated reactivate or reopen endpoint.
Archive and unarchive share `PUT /api/v1/{customer_id}/projects/{project_id}/archive_project`.
The body sets `data.attributes.status`: `9` archives the project and `1` unarchives (reactivates) it.

## Open points the document does not settle

The concrete list of which copy aspects and which attributes to reuse for a Daylite-driven create is a product decision, deferred to BL-037.
Custom field semantics for the Daylite link are a Daylite concern, not covered here.
