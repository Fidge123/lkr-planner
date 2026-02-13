# LKR Planner Completed Backlog

This file contains backlog items that have been successfully implemented and verified.

## EPIC 1: Project Hygiene and Architecture Basis

### BL-001: Protect OpenAPI files from commit ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: S

Scope:
- Extend `.gitignore` so that local OpenAPI artifacts in `docs/` are not accidentally committed (e.g., `docs/*openapi*.json`).

Acceptance Criteria:
- ✅ `git status` no longer shows OpenAPI files as new files.
- ✅ `docs/BACKLOG.md` remains versioned.

**Implementation:**
- Added pattern `docs/*openapi*.json` to `.gitignore`
- Verified: `daylite-openapi.json` and `planradar-openapi.json` are ignored
- Verified: `docs/BACKLOG.md` remains versioned

### BL-002: Unify Test Workflow ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: S

Scope:
- Add clear test scripts to `package.json` (`test`, optional `test:watch`).
- Add standardized local quality flow to README (`bun test`, `bun lint`, `bun format:check`).

Acceptance Criteria:
- ✅ `bun test` runs as the official standard command.
- ✅ Workflow is documented identically for agent and developer.

Tests (write first):
- At least one existing test run must run in CI/Locally with `bun test`.

**Implementation:**
- Added `test` and `test:watch` scripts to `package.json`
- Expanded README with Development section including full local quality workflow
- Verified: `bun test` and `bun run test` work identically
- Verified: All quality checks (`test`, `lint`, `format:check`) are documented

### BL-003: Tighten Integration Architecture (Frontend <-> Tauri Commands) ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: M

Scope:
- Define clear interface:
  - React UI only consumes service functions.
  - Network and Secrets run in Tauri/Rust Commands.
- Create folder structure for integrations (e.g., `src/services`, `src-tauri/src/integrations`).

Acceptance Criteria:
- ✅ At least one exemplary flow already uses the defined interface.
- ✅ Documented architecture note in repo (`docs/ARCHITECTURE.md`).

Tests (write first):
- ✅ Unit test for service facade in frontend (mock on command call).

**Implementation:**
- Created folder structure `src/services` and `src-tauri/src/integrations`.
- Implemented exemplary `HealthService` (TS) and `check_health` command (Rust).
- Created `docs/ARCHITECTURE.md` with principles and data flow documentation.
- Successfully executed unit tests in `src/services/health.spec.ts` and Rust tests in `health.rs`.

### BL-023: Transition Architecture Documentation to ADRs ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: S

Scope:
- Create `docs/adr` directory.
- Move current `ARCHITECTURE.md` content into initial Architecture Decision Records (ADRs).
- Update `AGENTS.md` to ensure future architecture decisions are documented as ADRs.
- Delete `ARCHITECTURE.md` after transition.

Acceptance Criteria:
- ✅ `docs/adr` contains initial ADRs.
- ✅ `AGENTS.md` mentions ADR requirement.
- ✅ `ARCHITECTURE.md` is removed.

**Implementation:**
- Added `docs/adr/0001-frontend-backend-separation.md`.
- Added `docs/adr/0002-service-facade-and-integration-structure.md`.
- Updated `AGENTS.md` with ADR documentation requirement (`docs/adr`).
- Removed `docs/ARCHITECTURE.md`.

### BL-024: CI/CD: Include Rust Tests ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: S

Scope:
- Add a step to the appropriate GitHub Action (e.g., `test.yml`) to run Rust tests using `cargo test`.
- Ensure the workflow fails if Rust tests fail.

Acceptance Criteria:
- ✅ GitHub Action runs `cargo test`.
- ✅ Build fails on failing Rust tests.

**Implementation:**
- Updated `.github/workflows/test.yml` to install Rust (`dtolnay/rust-toolchain@stable`).
- Added explicit Rust test step: `cargo test --manifest-path src-tauri/Cargo.toml`.
