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
Known attributes include name, street, zipcode, city, country, description, start date and end date.

Copying a source project uses `POST /api/v1/{customer_id}/projects/{project_id}/copy_project`.
This is the same copy feature offered in the Planradar UI.
It takes a new `name` plus boolean toggles that select which aspects to copy:
details, groups, ticket_types (forms), users, and components (layers).
The copy is performed server-side, so field-level edits happen afterward via project update.

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
