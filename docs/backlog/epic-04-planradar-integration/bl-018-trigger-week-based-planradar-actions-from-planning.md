# BL-018: Trigger Week-Based Planradar Actions from Planning

## Scope
- Create/reactivate projects assigned for the current week in Planradar.
- Trigger manually only in v1.

## Acceptance Criteria
- Action is traceably logged.
- Failed entries are individually re-executable.
- No automatic trigger runs in v1 (no week-change/background auto-sync).

## Tests (write first)
- Tests for trigger logic (current week only).
- Tests ensuring no automatic trigger path is active.
