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

### BL-026: Release GH Action with Timestamped Version on `main` ✅
**Status:** Completed (2026-02-13)  
Priority: P1  
Effort: S

Scope:
- Update the release GitHub Action to generate a unique timestamped version for every `main` release run.
- Append UTC timestamp to the base app version as prerelease segment (e.g. `0.1.0-main.20260213T153045Z`).
- Apply the computed version consistently to all release artifacts/metadata used by Tauri updater.
- Keep source-controlled base versions unchanged; stamping happens in CI only.

Acceptance Criteria:
- ✅ Two release runs from `main` always produce different version strings, even without code-level version bump.
- ✅ Generated version is valid SemVer and sortable in chronological order.
- ✅ GitHub release tag/name and updater metadata contain the stamped version.

Tests (write first):
- ✅ Added `scripts/stamp-release-version.spec.ts` for version format, validity, and uniqueness.

**Implementation:**
- Added `scripts/stamp-release-version.ts` to compute and validate stamped versions and patch `src-tauri/tauri.conf.json` during CI runtime only.
- Updated `.github/workflows/release.yml` to run stamping step (`id: stamp_version`) and use `steps.stamp_version.outputs.stamped_version` for release tag and name.

### BL-025: Introduce tauri-specta for typesafe commands ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: S

Scope:
- Add `specta` and `tauri-specta` dependencies to the Rust backend.
- Set up generator for TypeScript types in a shared location (e.g., `src-tauri/src/bindings.ts`).
- Migrate existing `check_health` command to use Specta.
- Update `HealthService` to use the generated bindings instead of manual `invoke`.

Acceptance Criteria:
- ✅ TypeScript bindings are automatically generated.
- ✅ `HealthService` uses generated types.
- ✅ No manual `invoke` calls for commands registered with Specta.

Tests (write first):
- ✅ Updated `src/services/health.spec.ts` to mock generated command bindings.

**Implementation:**
- Added Rust dependencies in `src-tauri/Cargo.toml`: `specta`, `specta-typescript`, and `tauri-specta`.
- Annotated `check_health` and health types in `src-tauri/src/integrations/health.rs` for Specta type generation.
- Replaced manual `tauri::generate_handler!` wiring with `tauri_specta::Builder` in `src-tauri/src/lib.rs`, including TS export to `src/generated/tauri.ts`.
- Updated `src/services/health.ts` to use generated binding commands (`commands.checkHealth`) and generated type aliases instead of direct `invoke`.

## EPIC 2: Domain Model and Local Storage

### BL-004: Define Domain Types for Planning v1 ✅
**Status:** Completed (2026-02-13)  
Priority: P0  
Effort: M

Scope:
- Add types for:
  - `Project` (Daylite reference, name, status)
  - `Employee` (skills, home location, primary iCal URL, absence iCal URL, active flag)
  - `Assignment` (Employee, project, period, source, sync status)
  - `SyncIssue` (Source, code, message, timestamp)

Acceptance Criteria:
- ✅ Dummy data migrated to new types.
- ✅ No `any`-based workarounds.

Tests (write first):
- ✅ Added `src/domain/planning.spec.ts` with unit tests for central guards/mappers.

**Implementation:**
- Added new planning domain module at `src/domain/planning.ts` with strict domain types and type guards.
- Added Daylite-to-domain mappers for project/contact records aligned with local OpenAPI shapes.
- Migrated dummy planning data to `Employee`, `Project`, and `Assignment` domain types in `src/data/dummy-data.ts`.
- Updated timetable components to consume the migrated typed data model.
- Removed obsolete legacy `src/types.ts`.

### BL-005: Build Local Configuration and Cache Store ✅
**Status:** Completed (2026-02-13)  
Priority: P1  
Effort: M

Scope:
- Persistence for local app configuration (file backend) for:
  - API endpoints
  - Tokens/references
  - Employee-specific settings
  - Project proposal filters (pipelines, columns, categories, exclusion status)
  - Contact filter for active employees (Default keyword: `Monteur`)
  - Routing settings for `openrouteservice.org` (API key, profile)
- Optional local cache for recently loaded Daylite data (without source-of-truth role).

Acceptance Criteria:
- ✅ Restart-safe loading/saving.
- ✅ Error cases provide German user message and technical debug details.

Tests (write first):
- ✅ Added Rust unit tests in `src-tauri/src/integrations/local_store.rs` for:
  - load defaults when file is missing
  - save/load restart safety
  - corrupt JSON file error
  - missing fields error

**Implementation:**
- Added new integration module `src-tauri/src/integrations/local_store.rs` with typed `LocalStore` schema and defaults.
- Implemented Tauri commands `load_local_store` and `save_local_store` and registered them in `src-tauri/src/lib.rs`.
- Persisted store in Tauri `app_config_dir` as `local-store.json`.
- Implemented structured `StoreError` with `code`, `userMessage`, and `technicalMessage`.
- Added ADR `docs/adr/0005-local-config-and-cache-store.md` for the storage decision.
