# ADR 0003: Planning Domain Model and Daylite Mappers

- Status: Accepted
- Date: 2026-02-13

## Context

The planning UI still relied on legacy ad-hoc types (`Employee`, `WorkItem`) that were not aligned with v1 backlog requirements.
Upcoming integrations require a stable domain model for projects, employees, assignments, and sync issues with explicit Daylite references and typed mapping boundaries.

## Decision

Introduce a central planning domain module at `src/domain/planning.ts` containing:
- Domain types: `Project`, `Employee`, `Assignment`, `SyncIssue`
- Runtime guards for key records
- Daylite record mappers for project/contact payloads based on `docs/daylite-openapi.json` and the Daylite API docs

Migrate dummy planning data in `src/data/dummy-data.ts` to use these domain types directly (`employees`, `projects`, `assignments`) and derive UI cell items from assignments.

## Consequences

- Domain model usage is consistent and does not rely on `any`.
- Mapper/guard behavior is unit-tested and provides a safer boundary for BL-006/BL-008 API integration tasks.
- UI dummy data now reflects the target planning model and reduces future migration effort.
