## Context

Planradar projects can be archived/closed. Linked projects that are archived need to be reopened before they can be used for assignments.

## Goals / Non-Goals

**Goals:**
- Detect archived linked projects
- Reactivate archived projects via API
- Log actions for audit and debugging
- Skip already active projects idempotently

**Non-Goals:**
- Creating new projects (handled in BL-037)
- Automatic reactivation on every sync (explicit action only)

## Decisions

### Detection Method
**Decision**: Check project status via Planradar API
- Fetch project details using linked ID
- Check `status` or `archived` field
- Differentiate: active, archived, closed states

### Reactivation Trigger
**Decision**: Explicit user action or pre-sync automation
- User can manually trigger reactivation from UI
- Optional: pre-sync hook reactivates before assignment sync
- Not automatic on every view, only when needed

### Idempotency
**Decision**: No-op for already active projects
- Check status before calling reactivate API
- If active, return success without API call
- If archived, call reactivate and return result
- If not found, return error with guidance

### Logging
**Decision**: Log to sync event log
- Log reactivation attempt with project ID
- Log success with timestamp
- Log failure with error message and stack trace
- Include project name for readability

## Risks / Trade-offs

- **Risk**: User lacks permission to reactivate in Planradar
  - **Mitigation**: Check permissions; show error with guidance

- **Risk**: Project was deleted (not just archived)
  - **Mitigation**: Distinguish not-found from archived; suggest BL-037

- **Risk**: Race condition with other Planradar users
  - **Mitigation**: Refresh status after reactivation; handle concurrent archive