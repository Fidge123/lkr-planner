## Context

BL-017 writes assignments to CalDAV inline, one VEVENT per assignment, currently with a fixed 08:00-16:00 window (see `build_ical_payload` in `src-tauri/src/integrations/calendar/ical.rs`).
Multiple same-day assignments therefore overlap.
There is no batch export step, so slot allocation must be applied at write time across the affected day.

## Goals / Non-Goals

**Goals:**
- A deterministic pure function that splits the fixed window into non-overlapping slots for N assignments
- Re-allocate and persist slots on every assignment create/update/delete
- Stable output for the same assignment set regardless of input order

**Non-Goals:**
- User-defined custom working-time windows
- Priority/duration-weighted slot sizes (all slots are equal)
- Conflict detection/resolution between assignments
- Re-slotting bare/absence/holiday events (only lkr-planner assignments)

## Decisions

### Time Window
**Decision**: Fixed `08:00-16:00` local (floating) time window, matching the existing `build_ical_payload` output.

### Slot Splitting Algorithm
**Decision**: Equal division of the 480-minute window by assignment count.
- Per-slot duration: 480 / n minutes
- Slots are [start, end) half-open intervals so adjacent slots touch but do not overlap
- Minute granularity (the 3-assignment case yields 08:00-10:40-13:20-16:00)

### Ordering Rule
**Decision**: Sort by VEVENT UID (string comparison) for a stable canonical order independent of input sequence.
UIDs are immutable, so repeated runs produce identical output.

### Where Re-allocation Runs
**Decision**: Re-allocation runs in the Rust backend as part of the BL-017 write commands, consistent with "all third-party API logic lives in the backend".
- On create/update/delete, the backend determines the affected employee + day(s), gathers that day's lkr-planner assignment VEVENTs, runs the pure allocation, and PUTs the updated DTSTART/DTEND for each event whose slot changed.
- An update that changes the day re-allocates both the source and target day.
- The pure `allocate_slots` function is unit-tested in isolation; the orchestration is covered by the BL-017 write-path tests.

### Identifying lkr-planner Assignments
**Decision**: Only events whose DESCRIPTION first line is a `daylite:/<path>` reference are re-slotted (same classification used on the read side). Bare, absence, and holiday events are never modified.

## Risks / Trade-offs

- **Risk**: Re-slotting issues extra CalDAV PUTs per write (one per same-day assignment).
  - **Mitigation**: Typical day has 1-5 assignments; only events whose slot actually changed need a PUT.

- **Risk**: A partial failure mid-redistribution leaves the day with mixed old/new slots.
  - **Mitigation**: Surface a German error; re-running any write on that day re-allocates deterministically and converges.

- **Risk**: Very large number of assignments (>10) creates tiny slots.
  - **Mitigation**: Documented limitation; assume typical day has 1-5 assignments.

- **Risk**: Daylight saving time transitions.
  - **Mitigation**: Use floating local time consistently, matching existing VEVENT output.
