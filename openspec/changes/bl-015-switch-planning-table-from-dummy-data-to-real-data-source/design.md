## Context

The weekly planning view uses hardcoded assignment fixtures. Users cannot persist their planning across app restarts. This change connects the planning table to the existing persistent storage (local store or backend).

## Goals / Non-Goals

**Goals:**
- Replace all dummy assignment data with persisted data
- Maintain existing loading/empty/error states in German
- Support week-to-week navigation with persistent data

**Non-Goals:**
- Assignment modal behavior (covered by BL-016, BL-031, BL-032, BL-033)

## Decisions

### Storage Approach
**Decision**: Use existing local JSON store in Tauri backend
- Leverage existing persistence infrastructure (BL-005)
- Assignments stored per employee/week key
- Efficient loading with caching

### Data Model
**Decision**: Simple employee-day-project mapping
- Key: `${employeeId}-${weekStart}-${dayOfWeek}`
- Value: Project reference (Daylite project ID or linked Planradar)

## Risks / Trade-offs

- **Risk**: Large number of assignments affecting performance
  - **Mitigation**: Use in-memory cache with TTL

- **Risk**: Schema migration for assignment data
  - **Mitigation**: Version the storage schema