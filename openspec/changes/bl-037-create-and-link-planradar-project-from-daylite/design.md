## Context

Users need to create Planradar projects for Daylite projects that don't have links. The flow should be seamless and idempotent to avoid creating duplicates.

## Goals / Non-Goals

**Goals:**
- Create new Planradar project from unlinked Daylite project
- Support template selection from existing projects
- Persist Planradar ID to Daylite custom field
- Ensure idempotent operation

**Non-Goals:**
- Reactivating archived linked projects (BL-038)
- Automatic project creation (user-initiated only)

## Decisions

### Creation Trigger
**Decision**: User initiates creation from project comparison UI
- Unlinked Daylite project shows "Create in Planradar" option
- User selects template project or starts blank

### Template Selection
**Decision**: Show list of existing Planradar projects as templates
- Allow filtering/search to find right template
- Default to blank if no template selected
- Copy name, description, and optionally custom fields from template

### Idempotency
**Decision**: Check for existing link before creation
- Query Daylite custom field for existing Planradar ID
- If ID exists and project exists in Planradar, skip creation
- Log warning if ID exists but project missing in Planradar (sync issue)

### Persistence
**Decision**: Write Planradar ID to Daylite custom field after creation
- Use Daylite API to update custom field
- Store mapping locally in sync state as backup
- Handle write failure gracefully (retry queue)

## Risks / Trade-offs

- **Risk**: Planradar API rate limiting during bulk creation
  - **Mitigation**: Add delay between creations; batch if API supports

- **Risk**: Custom field doesn't exist in Daylite
  - **Mitigation**: Detect missing field; prompt user to create it

- **Risk**: User cancels during creation
  - **Mitigation**: Partial state cleaned up; no orphan Planradar projects