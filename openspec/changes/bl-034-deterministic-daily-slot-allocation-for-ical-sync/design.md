## Context

When exporting employee assignments to iCal, multiple assignments on the same day need time slots. The iCal format requires specific start/end times for each event. We need a deterministic algorithm that produces stable output for the same input.

## Goals / Non-Goals

**Goals:**
- Allocate non-overlapping time slots within fixed window
- Ensure deterministic output for same assignment set
- Handle 1 to n assignments on one day
- Support reordered input producing canonical output

**Non-Goals:**
- User-defined custom working time windows
- Conflict detection/resolution (only allocation)
- Priority-based slot allocation

## Decisions

### Time Window
**Decision**: Fixed `08:00-16:00` local time window
- 8-hour window provides ample time for typical daily assignments
- Simple to understand and explain to users
- Can be made configurable in future if needed

### Slot Splitting Algorithm
**Decision**: Equal division of window by assignment count
- Window duration: 8 hours = 480 minutes
- Per-slot duration: 480 / n assignments
- Each slot has equal duration, no priority weighting

### Ordering Rule
**Decision**: Sort by assignment ID (string comparison)
- Stable canonical order regardless of input sequence
- Assignment ID is immutable, so order is deterministic
- Produces identical output on repeated sync runs

### Slot Boundaries
**Decision**: Slots are [start, end) half-open intervals
- First assignment: 08:00 to 08:00 + slot_duration
- Last assignment: 16:00 - slot_duration to 16:00
- Prevents overlap at boundary

## Risks / Trade-offs

- **Risk**: Very large number of assignments (>10) creates tiny slots
  - **Mitigation**: Document limitation; assume typical day has 1-5 assignments

- **Risk**: Daylight saving time transitions
  - **Mitigation**: Use local time consistently; DST gaps/overlaps handled by iCal library

- **Risk**: Equal slot sizes may not fit assignment durations
  - **Mitigation**: Slot allocation is for iCal display; actual assignment duration stored separately