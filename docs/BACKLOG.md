# LKR Planner Backlog

## Goal
Actionable, prioritized backlog for collaborative implementation with a coding agent.
Focus is on small, testable increments (Red-Green-Refactor) and clear acceptance criteria.

## Guidelines
- TDD in every task: first failing test, then minimal implementation, then refactor.
- UI texts always in German.
- API calls via `@tauri-apps/plugin-http`.
- New dependencies only after explicit decision (prepare options with Pros/Cons).
- Use OpenAPI files in `docs/` locally, but do not commit them.
- Daylite is the Source of Truth for projects and employees (local caches only as technical optimization).
- No offline support for v1 (online-first).

## Current Status (from codebase)
- Weekly view with dummy data exists (`src/app/*`, `src/data/dummy-data.ts`).
- Date helpers are partially tested (`src/app/util.spec.ts`).
- Daylite and Planradar integration is not yet implemented.
- Employee management is not yet implemented.

## Prioritized Epics
1. Project Hygiene and Architecture Basis
2. Domain Model and Local Storage
3. Daylite Integration
4. Planradar Integration
5. Employee Management
6. Planning Logic and Calendar Sync
7. Stability, Observability, Release

## Backlog Items

## EPIC 1: Project Hygiene and Architecture Basis

### BL-023: Transition Architecture Documentation to ADRs
Prioritized: P0  
Effort: S

Scope:
- Create `docs/adr` directory.
- Move current `ARCHITECTURE.md` content into initial Architecture Decision Records (ADRs).
- Update `AGENTS.md` to ensure future architecture decisions are documented as ADRs.
- Delete `ARCHITECTURE.md` after transition.

Acceptance Criteria:
- `docs/adr` contains initial ADRs.
- `AGENTS.md` mentions ADR requirement.
- `ARCHITECTURE.md` is removed.

### BL-024: CI/CD: Include Rust Tests
Prioritized: P0  
Effort: S

Scope:
- Add a step to the appropriate GitHub Action (e.g., `test.yml`) to run Rust tests using `cargo test`.
- Ensure the workflow fails if Rust tests fail.

Acceptance Criteria:
- GitHub Action runs `cargo test`.
- Build fails on failing Rust tests.

### BL-025: Introduce tauri-specta for typesafe commands
Prioritized: P0  
Effort: S

Scope:
- Add `specta` and `tauri-specta` dependencies to the Rust backend.
- Set up generator for TypeScript types in a shared location (e.g., `src-tauri/src/bindings.ts`).
- Migrate existing `check_health` command to use Specta.
- Update `HealthService` to use the generated bindings instead of manual `invoke`.

Acceptance Criteria:
- TypeScript bindings are automatically generated.
- `HealthService` uses generated types.
- No manual `invoke` calls for commands registered with Specta.


## EPIC 2: Domain Model and Local Storage

### BL-004: Define Domain Types for Planning v1
Priority: P0  
Effort: M

Scope:
- Add types for:
  - `Project` (Daylite reference, name, status)
  - `Employee` (Skills, location, iCal URL, active flag)
  - `Assignment` (Employee, project, period, source, sync status)
  - `SyncIssue` (Source, code, message, timestamp)

Acceptance Criteria:
- Dummy data migrated to new types.
- No `any`-based workarounds.

Tests (write first):
- Type/Unit tests for central mappers/guards.

### BL-005: Build Local Configuration and Cache Store
Priority: P1  
Effort: M

Scope:
- Persistence for local app configuration (e.g., Tauri store or file backend) for:
  - API endpoints
  - Tokens/references
  - Employee-specific settings
  - Project proposal filters (pipelines, columns, categories, exclusion status)
  - Contact filter for active employees (Default keyword: `Monteur`)
- Optional local cache for recently loaded Daylite data (without source-of-truth role).

Acceptance Criteria:
- Restart-safe loading/saving.
- Error cases provide German user message and technical debug details.

Tests (write first):
- Unit tests for load/save + error case (corrupt file, missing fields).

## EPIC 3: Daylite Integration

### BL-006: Daylite API Client (Basis)
Priority: P0  
Effort: M

Scope:
- Build minimal client for required endpoints:
  - Read/search projects
  - Read/search contacts (for employee mapping)
- Uniform error object including HTTP status.

Acceptance Criteria:
- Client returns typed responses.
- Errors are centrally normalized.

Tests (write first):
- Unit tests with mocked HTTP responses (200/401/429/500).

### BL-007: Daylite Project Synchronization (Read)
Priority: P0  
Effort: M

Scope:
- Load Daylite projects as Source of Truth.
- Map into internal `Project` model.
- Implement proposal logic for calendar view (locally configurable):
  - Pipeline rule: Pipeline `Aufträge` and column `Vorbereitung` or `Durchführung` (Defaults).
  - Category rule: Category `Überfällig` or `Liefertermin bekannt`.
  - Exclusion rule: Status `Done` is not proposed.

Acceptance Criteria:
- UI can load project list from Daylite (without dummy projects) and correctly filter proposal set.
- "Last synchronized" timestamp visible.
- Default rules are locally adjustable.

Tests (write first):
- Mapper tests for date/status fields.
- Service test for successful sync + API error.
- Rule tests for pipeline, category, and exclusion logic.

### BL-022: Project Search Outside Proposal Set
Priority: P0  
Effort: M

Scope:
- Provide a search in the assignment dialog that also finds projects not in the proposal set.
- Search via at least name + external reference (if available).

Acceptance Criteria:
- User can specifically find and assign a project even if it is not proposed.
- Search results are clearly distinguishable from proposed projects.

Tests (write first):
- UI tests for search input, result list, selection.
- Service tests for search API and error cases.

### BL-008: Use Daylite Contacts for Employee Configuration
Priority: P1  
Effort: M

Scope:
- Load contacts from Daylite and display as a possible employee source.
- Enable assignment Contact <-> local employee.
- Provide local configuration for contact filter (Default: keyword `Monteur`).

Acceptance Criteria:
- User can take over/assign contact as employee.
- Persisted assignment remains after restart.
- Filter changes take effect without code change.

Tests (write first):
- Test for contact-to-employee mapping.
- Test for persistence of assignment.

## EPIC 4: Planradar Integration

### BL-009: Planradar API Client (Basis)
Priority: P0  
Effort: M

Scope:
- Minimal client for:
  - Search/list projects
  - Create project (template-based, if needed)
  - Check project status (active/reopen)

Acceptance Criteria:
- Typed responses and standardized errors.
- Configurable tenant/account parameters.

Tests (write first):
- Unit tests analogous to Daylite client including Auth and Rate-Limit cases.

### BL-010: Daylite -> Planradar Project Comparison
Priority: P0  
Effort: L

Scope:
- Comparison logic:
  - Does a corresponding Planradar project exist (via Daylite custom field link)?
  - If no link exists: User can link an existing Planradar project or create a new project via cloning.
  - If linked Planradar project is archived/closed: automatically reactivate (unarchive/reopen).
- Persist the linked Planradar project reference as a custom field in Daylite.
- Ensure idempotency.

Acceptance Criteria:
- Multiple runs do not create duplicates.
- Every action is logged as a sync event.
- After successful linking, the link is stored in Daylite and reused in the next run.

Tests (write first):
- Service tests for cases: new, already existing, closed, API error.
- Test for persistence and reuse of the Daylite custom field link.

### BL-011: Mapping Rules Daylite Project -> Planradar Template
Priority: P1  
Effort: M

Scope:
- Configurable rule matrix (e.g., by project category/type).
- Fallback rule for unmapped projects.
- Make clone source selectable as template or existing Planradar project.

Acceptance Criteria:
- Ruleset is editable in UI (at least basic form).
- Missing mapping creates clear SyncIssue instead of hard-fail.
- Clone flow works for both variants (template, project).

Tests (write first):
- Rule Engine tests (hit, fallback, invalid rule).

## EPIC 5: Employee Management

### BL-012: Employee List with CRUD
Priority: P0  
Effort: M

Scope:
- Screen for employee management:
  - Create
  - Edit
  - Deactivate
  - Delete (with protection for active assignments)

Acceptance Criteria:
- Complete CRUD flow without reload.
- All texts and error messages in German.

Tests (write first):
- Unit tests for validation (mandatory fields, iCal URL format).
- UI tests for Create/Edit/Delete flows.

### BL-013: Model Skills, Availability and Location
Priority: P1  
Effort: M

Scope:
- Expand employee with structured skills, weekly availability and home location.

Acceptance Criteria:
- Data is maintained and persisted in the form.
- Planning view shows availability context (e.g., hint for absence).

Tests (write first):
- Tests for availability calculation per weekday.

### BL-014: Store and Validate iCal Source per Employee
Priority: P0  
Effort: M

Scope:
- Save iCal URL per employee.
- Basic validation + connection test (manually triggerable).

Acceptance Criteria:
- Invalid URLs are handled cleanly.
- Connection test provides clear success/error message.

Tests (write first):
- Parser/validation tests.
- Error case tests for unreachable calendar sources.

## EPIC 6: Planning Logic and Calendar Sync

### BL-015: Switch Planning Table from Dummy Data to Real Data Source
Priority: P0  
Effort: M

Scope:
- Decouple `dummy-data`, connect service layer instead.
- Add load, empty, and error states in weekly view.

Acceptance Criteria:
- Weekly view works with persistent data.
- Error states are understandable for users (German).

Tests (write first):
- UI tests for Loading/Empty/Error.

### BL-016: Create/Edit/Delete Assignments in Weekly View
Priority: P0  
Effort: L

Scope:
- Click on cell opens editor for assignment:
  - Select project
  - Visual warning symbol for projects without Planradar link
  - Action "Link or create Planradar project (Clone)"
  - Set period
  - Show conflicts
- Save changes persistently.

Acceptance Criteria:
- End-to-end flow for assignment CRUD exists.
- Conflicts are made visible before saving.
- Projects without Planradar link are clearly marked and directly editable in planning.

Tests (write first):
- Service tests for overlap detection.
- UI tests for Create/Edit/Delete.

### BL-017: iCal Synchronization for Employee Assignments
Priority: P0  
Effort: L

Scope:
- Mirror changes to assignments in employee iCal.
- Idempotent synchronization (no duplicate appointments).

Acceptance Criteria:
- New/Update/Delete in planning creates correct iCal action.
- Sync status per assignment viewable.

Tests (write first):
- Sync Service tests including retry scenarios.

### BL-018: Trigger Week-Based Planradar Actions from Planning
Priority: P1  
Effort: M

Scope:
- Create/reactivate projects assigned for the current week in Planradar.
- Trigger manually and optionally automatically on week change.

Acceptance Criteria:
- Action is traceably logged.
- Failed entries are individually re-executable.

Tests (write first):
- Tests for trigger logic (current week only).

## EPIC 7: Stability, Observability, Release

### BL-019: Central Error and Sync Issue Panel
Priority: P1  
Effort: M

Scope:
- UI area with last errors, warnings and sync issues.
- Filterable by source (Daylite, Planradar, iCal).

Acceptance Criteria:
- User can trace error cases and specifically re-trigger them.

Tests (write first):
- Reducer/State tests for event collection and filter.

### BL-020: Background Sync and Manual Synchronization
Priority: P1  
Effort: M

Scope:
- Manual "Sync Now" button.
- Optional interval sync with lock against parallel runs.

Acceptance Criteria:
- No competing sync runs.
- Visible feedback on running sync.

Tests (write first):
- Tests for run-lock and re-execution after error.

### BL-021: Release Hardening for macOS
Priority: P2  
Effort: M

Scope:
- Build checklist (`bun build:macos`, smoke test, signing/notarization as separate process if needed).
- Basic telemetry/logging for support cases (local).

Acceptance Criteria:
- Reproducible release process documented.
- Critical errors reconstructible from logs.

Tests (write first):
- Smoke test checklist as executable flow (manual + script where possible).

## Open Product Questions
1. How should conflict logic be prioritized: hard block or warning?
2. Which conflict types should be considered in v1:
   - Double booking of an employee in the same period
   - Absence according to iCal
   - Skill mismatch
   - Location/travel conflict
3. Should Planradar sync in v1 run only manually or also automatically (e.g., on week change/background)?
4. What is the name of the Daylite custom field for the Planradar link (technical key), and is it already present?
