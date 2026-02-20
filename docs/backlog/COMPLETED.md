# LKR Planner Completed Backlog

This file stores concise implementation summaries for completed or reverted backlog items.
It is intended for understanding past decisions, not for detailed task specifications.

## EPIC 1: Project Hygiene and Architecture Basis

### BL-001: Protect OpenAPI files from commit
- Status: Completed (2026-02-13)
- Summary: Added ignore rules for local OpenAPI artifacts in `docs/` so generated API files are not accidentally committed.

### BL-002: Unify Test Workflow
- Status: Completed (2026-02-13)
- Summary: Standardized project test scripts in `package.json` and aligned README quality workflow around `bun test`, linting, and formatting.

### BL-003: Tighten Integration Architecture (Frontend <-> Tauri Commands)
- Status: Completed (2026-02-13)
- Summary: Established service-facade boundaries between React and Tauri commands and introduced integration folder structure in Rust.

### BL-023: Transition Architecture Documentation to ADRs
- Status: Completed (2026-02-13)
- Summary: Replaced architecture narrative document with ADR-based architecture tracking under `docs/adr` and updated agent guidance.

### BL-024: CI/CD: Include Rust Tests
- Status: Completed (2026-02-13)
- Summary: Extended CI pipeline to run Rust tests with `cargo test` and fail builds on backend test regressions.

### BL-026: Release GH Action with Timestamped Version on `main`
- Status: Completed (2026-02-13)
- Summary: Implemented CI-side timestamped version stamping for unique `main` release artifacts and updater metadata.

### BL-025: Introduce tauri-specta for typesafe commands
- Status: Completed (2026-02-13)
- Summary: Migrated command wiring to Specta-generated TypeScript bindings and removed manual frontend command typing for covered commands.

## EPIC 2: Domain Model and Local Storage

### BL-004: Define Domain Types for Planning v1
- Status: Completed (2026-02-13)
- Summary: Introduced typed planning domain model and Daylite guards/mappers, then migrated dummy planning data to the new model.

### BL-005: Build Local Configuration and Cache Store
- Status: Completed (2026-02-13)
- Summary: Added typed restart-safe local JSON store in Tauri with structured German/user-facing and technical error payloads.

## EPIC 3: Daylite Integration

### BL-006: Daylite API Client (Basis)
- Status: Completed (2026-02-14)
- Summary: Implemented typed Daylite project/contact read/search commands with normalized error handling and refresh-token rotation persistence.

### BL-007: Daylite Project Loading with Short-Lived Cache (Read)
- Status: Completed (2026-02-15)
- Summary: Replaced dummy project loading with Daylite-backed reads using in-memory TTL cache, request coalescing, and stale-data fallback UX.

### BL-008: Use Daylite Contacts for Employee Configuration
- Status: Completed (2026-02-20)
- Summary: Switched employee source to Daylite contacts (`Monteur` category), standardized iCal handling to contact `urls`, and added Daylite write command support.

### BL-028: Standard-Filter Logic for Daylite Projects
- Status: Reverted (2026-02-20)
- Summary: Implemented Standard-Filter defaults and terminology updates, then reverted this direction as planning requirements changed.

### BL-030: Show Daylite-Projects below the planning table
- Status: Completed (2026-02-15)
- Summary: Added a Daylite project overview section below the planning table using existing in-memory data without extra backend requests.
