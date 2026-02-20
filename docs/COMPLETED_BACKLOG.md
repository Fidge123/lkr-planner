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
  - Standard-Filter (pipelines, columns, categories, exclusion status)
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

## EPIC 3: Daylite Integration

### BL-006: Daylite API Client (Basis) ✅
**Status:** Completed (2026-02-14)  
Priority: P0  
Effort: M

Scope:
- Build minimal client for required endpoints:
  - Read/search projects
  - Read/search contacts (for employee mapping)
- Uniform error object including HTTP status.

Acceptance Criteria:
- ✅ Client returns typed responses.
- ✅ Errors are centrally normalized.

Tests (write first):
- ✅ Added Rust unit tests in `src-tauri/src/integrations/daylite.rs` with mocked HTTP responses:
  - success (200)
  - unauthorized (401)
  - rate limit (429)
  - server error (500)
- ✅ Added token refresh/rotation test for Daylite access/refresh token flow.

**Implementation:**
- Added new Daylite integration module `src-tauri/src/integrations/daylite.rs`.
- Implemented typed API client methods for project/contact list and search endpoints.
- Implemented centralized `DayliteApiError` with machine-readable code, optional HTTP status, German user message, and technical details.
- Implemented Daylite access/refresh token flow via `/personal_token/refresh_token`.
- Added persistent tracking of rotated Daylite `access` + `refresh` tokens in local store (`tokenReferences.dayliteAccessToken`, `tokenReferences.dayliteRefreshToken`).
- Added Tauri commands for Daylite connect/list/search flows and registered them in `src-tauri/src/lib.rs`.
- Added ADR `docs/adr/0006-daylite-access-refresh-token-rotation.md`.

### BL-007: Daylite Project Loading with Short-Lived Cache (Read) ✅
**Status:** Completed (2026-02-15)  
Priority: P0  
Effort: M

Scope:
- Replace project dummy data in planning flows with Daylite project reads on demand.
- Add short-lived local cache with default TTL `30s` and request coalescing.
- Show German loading/error/retry UI states and keep last successful data usable on transient errors.

Acceptance Criteria:
- ✅ Planning UI loads project data from Daylite (no dummy project list in active flow).
- ✅ Repeated access within `30s` reuses cached data.
- ✅ Requests after TTL expiry fetch fresh Daylite data.
- ✅ On fetch error, UI shows German message and supports retry while retaining previous data.

Tests (write first):
- ✅ Added frontend service tests in `src/services/daylite-projects.spec.ts` for:
  - mapping normalization (status/date)
  - TTL cache hit/miss
  - in-flight request coalescing
  - stale-cache fallback on transient error
  - no-cache error path
- ✅ Added planning UI state tests in `src/app/page.spec.tsx` for loading/error/loaded rendering.
- ✅ Added Rust Daylite malformed payload test in `src-tauri/src/integrations/daylite.rs`.

**Implementation:**
- Added frontend Daylite project loader service with `30s` in-memory cache and coalescing in `src/services/daylite-projects.ts`.
- Added planning hook `src/app/use-planning-projects.ts` and wired it into `src/app/page.tsx`.
- Updated timetable rendering to resolve assignment labels from loaded Daylite projects (`src/app/components/timetable-row.tsx`, `src/data/dummy-data.ts`).
- Extended Daylite project summary payload fields and binding types (`src-tauri/src/integrations/daylite.rs`, `src/generated/tauri.ts`).
- Added ADR `docs/adr/0007-daylite-project-on-demand-loading-cache.md`.

### BL-008: Use Daylite Contacts for Employee Configuration ✅
**Status:** Completed (2026-02-20)  
Priority: P1  
Effort: M

Scope:
- Load Daylite contacts and use only category `Monteur` as employee source.
- Read and write two iCal references via Daylite contact `urls`:
  - Primary assignment iCal URL
  - Secondary absence iCal URL (vacation/sick leave)
- Use persisted cache values to show employees immediately after restart.

Acceptance Criteria:
- ✅ Correct employees are shown from Daylite contact category `Monteur`.
- ✅ Restart-safe cached values are used for immediate employee rendering.
- ✅ Both iCal URLs are readable and writable through Daylite contact `urls`.

Tests (write first):
- ✅ Added mapping/filter tests in `src/services/daylite-contacts.spec.ts` for contact-to-employee mapping and `Monteur` category filtering.
- ✅ Added iCal read/write mapping tests:
  - `src/domain/planning.spec.ts` (urls-only iCal extraction and url upsert mapping)
  - `src/services/daylite-contacts.spec.ts` (command payload for iCal write)
- ✅ Added planning UI test in `src/app/page.spec.tsx` to verify Daylite-backed employee rendering.
- ✅ Added Rust Daylite client test in `src-tauri/src/integrations/daylite/client.rs` for PATCH update of contact iCal urls.

**Implementation:**
- Added Daylite employee/contact service with in-memory TTL cache, persisted local-cache read/write support, and category filtering in `src/services/daylite-contacts.ts`.
- Added `usePlanningEmployees` hook and wired planning table employees to Daylite contacts in `src/app/use-planning-employees.ts` and `src/app/page.tsx`.
- Implemented Daylite contact iCal write command (`daylite_update_contact_ical_urls`) and full-record contact loading in Rust (`src-tauri/src/integrations/daylite/contacts.rs`, `src-tauri/src/integrations/daylite/client.rs`, `src-tauri/src/lib.rs`).
- Expanded local store contact cache payload for restart rendering (`src-tauri/src/integrations/local_store.rs`, `src/generated/tauri.ts`).
- Removed `extra_fields` iCal mapping path and standardized to Daylite `urls` mapping only (`src/domain/planning.ts`).
- Added ADR `docs/adr/0008-daylite-employee-contacts-and-urls-ical.md`.

### BL-028: Standard-Filter Logic for Daylite Projects ✅
**Status:** Reverted (2026-02-20)
Priority: P0  
Effort: S

Scope:
- Replace term "proposal set" with:
  - `Standard-Filter` (saved default rule set)
  - `Filter` (currently active rule set in UI context)
- Implement Standard-Filter rule engine for Daylite projects:
  - Pipeline rule default: pipeline `Aufträge` and column `Vorbereitung` or `Durchführung`
  - Category rule default: category `Überfällig` or `Liefertermin bekannt`
  - Exclusion rule default: status `Done` is never shown
- Apply Standard-Filter by default in planning/assignment project lists.

Acceptance Criteria:
- ✅ Project lists use Standard-Filter by default.
- ✅ `Done` projects are excluded even if other rules match.
- ✅ Empty result after filtering shows clear German state text (`Keine Projekte im Standard-Filter gefunden`).

Tests (write first):
- ✅ Added rule engine tests in `src/services/standard-filter.spec.ts` for default rules and exclusion precedence.
- ✅ Added integration tests in `src/services/daylite-projects.spec.ts` for Standard-Filter application on loaded Daylite projects.
- ✅ Added UI empty-state assertion in `src/app/page.spec.tsx` for Standard-Filter with no results.

**Implementation:**
- Added Standard-Filter rule engine in `src/services/standard-filter.ts` with documented defaults and exclusion precedence.
- Applied Standard-Filter by default in Daylite project loading flow (`src/services/daylite-projects.ts`) for network, cache, and stale-cache paths.
- Updated planning project empty-state text to `Keine Projekte im Standard-Filter gefunden` in `src/app/page.tsx`.
- Renamed local store terminology from proposal filters to Standard-Filter (`src-tauri/src/integrations/local_store.rs`, `src/generated/tauri.ts`) with backward-compatible alias support for existing `projectProposalFilters` payloads.

### BL-030: Show Daylite-Projects below the planning table ✅
**Status:** Completed (2026-02-15)  
Priority: P0  
Effort: S

Scope:
- Add a project overview section below the weekly planning table.
- Render already loaded Daylite projects (no extra backend request path).
- Display at least project name, status, and due date in German UI.

Acceptance Criteria:
- ✅ Loaded Daylite projects are visible below the planning table.
- ✅ Section uses the existing in-memory dataset from planning project state.
- ✅ Rendering the section does not create additional Daylite requests.
- ✅ German loading/empty states are visible (`Projekte werden geladen...`, `Keine Projekte im Standard-Filter gefunden`).

Tests (write first):
- ✅ Added/updated UI tests in `src/app/page.spec.tsx` for:
  - loading state in the overview section
  - empty state below table
  - loaded row rendering including status and due date
- ✅ Added service/UI integration-oriented cache test in `src/services/daylite-projects.spec.ts` for secondary UI consumer reuse without extra request.

**Implementation:**
- Added loaded-project overview UI under the table in `src/app/page.tsx`.
- Added German status mapping and due-date formatting for overview rows in `src/app/page.tsx`.
- Kept existing top-level error banner behavior and avoided duplicate error panels.
