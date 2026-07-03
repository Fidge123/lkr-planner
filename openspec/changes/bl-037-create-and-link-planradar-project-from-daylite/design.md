## Context

Users need to create Planradar projects for Daylite projects that don't have links. The flow should be seamless and idempotent to avoid creating duplicates.

## Goals / Non-Goals

**Goals:**
- Create new Planradar project from unlinked Daylite project
- Support source project selection from existing projects (copy-project then edit)
- Persist Planradar ID to the `planradar-link` Daylite custom field
- Ensure idempotent operation

**Non-Goals:**
- Reactivating archived linked projects (BL-038)
- Automatic project creation (user-initiated only)

## Decisions

### Creation Trigger
**Decision**: User initiates creation from project comparison UI
- Unlinked Daylite project shows "Create in Planradar" option
- User selects template project or starts blank

### Source Project Selection and copy flow
**Decision**: Use the Planradar copy-project endpoint, then edit (hybrid flow)
- Source-based creation uses `copy_project` (see BL-009) rather than a manual read-then-recreate, matching the native Planradar copy feature
- Show a list of existing Planradar projects to use as a source, with filtering/search
- The user picks a name and which aspects to copy via the endpoint toggles: details, groups, ticket types (forms), users, components (layers)
- After the server-side copy, open an edit form to adjust the new project's details (address, dates, description) via `PUT projects/{id}` before finishing
- Default to a blank project (Daylite name only) if no source project is selected

### Idempotency
**Decision**: Check for existing link before creation
- Query Daylite custom field for existing Planradar ID
- If ID exists and project exists in Planradar, skip creation
- Log warning if ID exists but project missing in Planradar (sync issue)

### Persistence
**Decision**: Write Planradar ID to the `planradar-link` Daylite custom field after creation
- Use Daylite API to update the custom field
- Store mapping locally in sync state as backup
- Handle write failure gracefully (retry queue)

## Risks / Trade-offs

- **Risk**: Planradar API rate limiting during bulk creation
  - **Mitigation**: Add delay between creations; batch if API supports

- **Risk**: Custom field doesn't exist in Daylite
  - **Mitigation**: App creates the fixed `planradar-link` field automatically if missing

- **Risk**: User cancels during creation
  - **Mitigation**: Partial state cleaned up; no orphan Planradar projects