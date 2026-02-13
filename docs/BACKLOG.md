# LKR Planner Backlog

## Guidelines
- TDD in every task: first failing test, then minimal implementation, then refactor.
- UI texts always in German.
- API calls via `@tauri-apps/plugin-http`.
- New dependencies only after explicit decision (prepare options with Pros/Cons).
- Use OpenAPI files in `docs/` locally, but do not commit them.
- Daylite is the Source of Truth for projects and employees (local caches only as technical optimization).
- No offline support for v1 (online-first).

## Backlog Items

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
- Maintain two iCal references in Daylite contact mapping:
  - Primary assignment iCal URL
  - Secondary absence iCal URL (vacation/sick leave)

Acceptance Criteria:
- User can take over/assign contact as employee.
- Persisted assignment remains after restart.
- Filter changes take effect without code change.
- Both iCal URLs are readable/writable through Daylite contact data.

Tests (write first):
- Test for contact-to-employee mapping.
- Test for persistence of assignment.
- Tests for mapping both iCal URLs from/to Daylite contact fields.

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
- Use configurable Daylite field mapping for this link:
  - Default field label: `Planradar-Projekt-ID`
  - Stored field value: `planradarProjectId` returned by Planradar API
  - Daylite field key/id is metadata to locate the field, not the stored project id itself
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
- Save two iCal URLs per employee (from Daylite):
  - Primary assignment iCal
  - Secondary absence iCal (vacation/sick leave)
- Basic validation + connection test (manually triggerable).

Acceptance Criteria:
- Invalid URLs are handled cleanly.
- Connection test provides clear success/error message.
- Absence iCal can be validated and tested independently from primary iCal.

Tests (write first):
- Parser/validation tests.
- Error case tests for unreachable calendar sources.
- Tests for separate validation/reporting of primary vs absence iCal.

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

### BL-027: Import German Holidays via Nager API for Week View
Priority: P1  
Effort: M

Scope:
- Integrate public holiday import from `https://date.nager.at` for Germany (`DE`).
- Fetch holidays per year from Nager API and map to app day model.
- In weekly view, indicate holidays that apply to at least one of:
  - Hamburg (`HH`)
  - Schleswig-Holstein (`SH`)
  - Mecklenburg-Vorpommern (`MV`)
- Show holiday indication in day header (German label/icon) and keep normal planning interactions.
- Cache yearly holiday data locally to avoid repeated API calls during week navigation.

Acceptance Criteria:
- Week view marks relevant holiday days for `HH`, `SH`, `MV`.
- If a holiday applies only to a subset of these states, the UI indicates affected states.
- Weeks crossing year boundaries correctly show holidays from both years.
- API failure does not break planning view; user gets a German warning state.

Tests (write first):
- Service tests for Nager API mapping/filtering (`DE`, `HH`/`SH`/`MV`).
- Tests for year-boundary fetch behavior (two-year query).
- UI tests for holiday indicator rendering in week header.
- Error-state test for Nager API timeout/failure.

### BL-016: Create/Edit/Delete Assignments in Weekly View
Priority: P0  
Effort: L

Scope:
- Click on cell opens editor for assignment:
  - Select project
  - Visual warning symbol for projects without Planradar link
  - Action "Link or create Planradar project (Clone)"
  - Set one or many workdays (non-contiguous days supported)
  - Show warnings (advisory only, never blocking save)
- Support multiple projects per employee/day and multiple employees per project.
- Save changes persistently.

Acceptance Criteria:
- End-to-end flow for assignment CRUD exists.
- Warnings are visible before saving, but do not block save.
- Projects without Planradar link are clearly marked and directly editable in planning.
- Warning behavior is implemented as:
  - High priority, non-dismissable: no linked Planradar project
  - Low priority, non-dismissable: any event in secondary absence iCal (vacation/sick leave)
  - Low priority, dismissable: skill mismatch
  - Low priority, dismissable: estimated driving time exceeds 2 hours
- Driving-time warning origin rule:
  - Default origin is employee home location (from Daylite contact)
  - If there is an earlier assignment on the same day, use that project location as origin
  - Only earlier same-day assignments may override home-location origin
- Driving-time warning source:
  - Use `openrouteservice.org` Directions API for travel-time estimation

Tests (write first):
- Service tests for warning evaluation and priority/dismissability rules.
- UI tests for Create/Edit/Delete.
- UI tests for warning rendering and dismiss action (dismissable only).
- Service tests for driving-time origin precedence (home vs earlier same-day project).
- Service tests with mocked `openrouteservice.org` responses (success, timeout, API error).

### BL-017: iCal Synchronization for Employee Assignments
Priority: P0  
Effort: L

Scope:
- Mirror changes to assignments in employee iCal.
- Idempotent synchronization (no duplicate appointments).
- Weekly view remains day-based (no exact time input).
- iCal events use a fixed daily dummy window `08:00-16:00` (local time).
- If an employee has multiple projects on the same day, split this window evenly across those projects.
- Use secondary absence iCal as warning input only; assignment events are not written to the absence calendar.

Acceptance Criteria:
- New/Update/Delete in planning creates correct iCal action.
- Sync status per assignment viewable.
- Slot splitting is deterministic and stable for repeated syncs.

Tests (write first):
- Sync Service tests including retry scenarios.
- Tests for same-day slot splitting (1..n assignments/day).

### BL-018: Trigger Week-Based Planradar Actions from Planning
Priority: P1  
Effort: M

Scope:
- Create/reactivate projects assigned for the current week in Planradar.
- Trigger manually only in v1.

Acceptance Criteria:
- Action is traceably logged.
- Failed entries are individually re-executable.
- No automatic trigger runs in v1 (no week-change/background auto-sync).

Tests (write first):
- Tests for trigger logic (current week only).
- Tests ensuring no automatic trigger path is active.

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

### BL-020: Manual Synchronization Runner (v1)
Priority: P1  
Effort: M

Scope:
- Manual "Sync Now" button.
- Sync run-lock to prevent parallel runs.

Acceptance Criteria:
- No competing sync runs.
- Visible feedback on running sync.
- No scheduled/background synchronization in v1.

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
- Keine offenen Produktfragen aktuell.
