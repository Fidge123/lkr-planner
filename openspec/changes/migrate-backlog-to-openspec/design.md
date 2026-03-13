## Context

The project currently uses a markdown-based backlog system in `docs/backlog/` with epics as folders and BLI (Backlog Item) markdown files. This system has served well but lacks:
- Structured workflow automation
- Artifact dependency tracking
- Integration with implementation tasks
- Standardized proposal-design-specs-tasks workflow

The openspec tool is already configured in the project and is being adopted as the new change management system.

## Goals / Non-Goals

**Goals:**
- Migrate all pending backlog items from `docs/backlog/` to openspec changes
- Set up the openspec workflow as the standard for future changes
- Maintain backlog item context and relationships during migration
- Archive the old backlog system after successful migration

**Non-Goals:**
- Migrate completed items from `docs/backlog/COMPLETED.md` (these remain as historical record)
- Modify any core application code
- Create detailed specifications for each backlog item (each openspec change will have its own artifacts)

## Decisions

### Migration Approach
**Decision**: Use a bulk migration approach - create one openspec change per backlog item
- Each BLI file becomes an openspec change with proposal/design/tasks artifacts
- Epics are discarded
- Completed items stay in COMPLETED.md as historical reference

### Change Naming
**Decision**: Use existing BLI IDs as the change name (kebab-case)
- Example: `bl-039-record-and-replay-http-testing` → `bl-039-record-and-replay-http-testing`
- This maintains traceability between old and new systems

### Artifact Structure
**Decision**: For each backlog item, create minimal openspec artifacts:
- `proposal.md`: Summary of the backlog item (what/why)
- `design.md`: Technical approach if needed
- `tasks.md`: Implementation tasks

## Risks / Trade-offs

- **Risk**: Some backlog items may lack sufficient detail for openspec artifacts
  - **Mitigation**: Review each item and enhance as needed during migration

## Migration Plan

1. Inventory all pending backlog items in `docs/backlog/`
2. For each backlog item:
   - Create new openspec change using `openspec new change`
   - Create proposal.md based on BLI content
   - Create design.md if technical decisions are documented
   - Create tasks.md with implementation steps
3. Verify all changes are created and apply-ready
4. Delete old backlog folder
