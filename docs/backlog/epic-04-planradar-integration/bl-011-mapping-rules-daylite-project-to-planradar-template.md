# BL-011: Mapping Rules Daylite Project -> Planradar Template

## Scope
- Configurable rule matrix (e.g., by project category/type).
- Fallback rule for unmapped projects.
- Make clone source selectable as template or existing Planradar project.

## Acceptance Criteria
- Ruleset is editable in UI (at least basic form).
- Missing mapping creates clear SyncIssue instead of hard-fail.
- Clone flow works for both variants (template, project).

## Tests (write first)
- Rule Engine tests (hit, fallback, invalid rule).
