# LKR Planner Backlog

## Guidelines
- TDD in every task: first failing test, then minimal implementation, then refactor.
- UI texts and user-facing text ALWAYS in German. Code, technical documentation and backlog in English.
- API calls via `@tauri-apps/plugin-http`.
- New dependencies only after explicit decision (prepare options with Pros/Cons).
- Use OpenAPI files in `docs/` locally, but do not commit them.
- Daylite is the Source of Truth for projects and employees (local caches only as technical optimization).
- No offline support for v1 (online-first).

## Epic Overview

### EPIC 3: Daylite Integration

This epic covers Daylite data retrieval capabilities needed by the assignment modal and planning workflows.
The main current focus is search behavior that supports assignment suggestions and explicit lookup.

Folder: `backlog/epic-03-daylite-integration`

### EPIC 4: Planradar Integration

This epic handles Planradar project linking and creation, but it is intentionally scheduled after the iCal baseline for v1.

Folder: `backlog/epic-04-planradar-integration`

### EPIC 5: Employee Management

This epic handles employee master data and iCal source setup used by the planning and synchronization flows.
For v1, employee iCal configuration has higher priority than extended skill/travel modeling.

Folder: `backlog/epic-05-employee-management`

### EPIC 6: Planning Logic and Calendar Sync

This epic defines the core planning experience: weekly grid behavior, assignment modal flow, calendar cell rendering, and iCal synchronization.
It includes holiday handling limited to Germany-wide and MV holidays for v1.

Folder: `backlog/epic-06-planning-logic-and-calendar-sync`

### EPIC 7: Stability, Observability, Release

This epic covers operational controls around synchronization behavior, error visibility, and reproducible macOS releases.

Folder: `backlog/epic-07-stability-observability-release`
