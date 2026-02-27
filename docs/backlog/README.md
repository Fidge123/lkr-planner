# LKR Planner Backlog

## Guidelines
- TDD in every task: first failing test, then minimal implementation, then refactor.
- UI texts and user-facing text ALWAYS in German. Code, technical documentation and backlog in English.
- New dependencies only after explicit decision (prepare options with Pros/Cons).
- Daylite is the Source of Truth for projects and employees (local caches only as technical optimization).
- YAGNI (You Ain't Gonna Need It): Avoid code that is not required for the current scope

## Epic Overview

### EPIC 3: Daylite Integration

This epic covers project-query capabilities required by the assignment modal.
It focuses on search/filtering and deterministic suggestion input data.

Folder: `backlog/epic-03-daylite-integration`

### EPIC 4: Planradar Integration

This epic handles Planradar link/create/reactivate flows and weekly manual execution.
It stays after iCal baseline scope for v1.

Folder: `backlog/epic-04-planradar-integration`

### EPIC 5: Employee Management

This epic is limited to employee iCal source validation and diagnostics.
Employee master data itself comes from Daylite contacts.

Folder: `backlog/epic-05-employee-management`

### EPIC 6: Planning Logic and Calendar Sync

This epic defines the core planning user flow: assignment modal behavior, cell item composition, and iCal write synchronization.
Large items were split into smaller BLIs to reduce delivery risk.

Folder: `backlog/epic-06-planning-logic-and-calendar-sync`
