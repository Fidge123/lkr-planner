## Context

This change enables linking Daylite projects to existing Planradar projects. It builds on the Planradar API client from BL-009 and uses Daylite's custom fields to store the link reference.

## Goals / Non-Goals

**Goals:**
- Detect existing links between Daylite and Planradar projects
- Allow manual linking of unlinked projects
- Persist links in Daylite custom fields
- Ensure idempotent operations

**Non-Goals:**
- Creating new Planradar projects (covered by BL-037)
- Reactivating archived projects (covered by BL-038)
- Automatic project matching

## Decisions

### Link Storage
**Decision**: Store Planradar project ID in Daylite custom field
- Uses existing Daylite custom field infrastructure
- Persists link data with the Daylite project
- Supports synchronization across devices

### Idempotency
**Decision**: Check for existing link before writing
- Query custom field before creating new link
- Reuse existing link if present to avoid duplicates

## Risks / Trade-offs

- **Risk**: Custom field not configured
  - **Mitigation**: Validate field exists before operations, show clear error

- **Risk**: Planradar project deleted after linking
  - **Mitigation**: Validate project exists when reading link