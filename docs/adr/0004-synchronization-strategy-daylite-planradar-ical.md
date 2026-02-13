# ADR 0004: Synchronization Strategy (Daylite, Planradar, iCal)

- Status: Accepted
- Date: 2026-02-13

## Context

The previous implementation assumptions were too broad and introduced unnecessary complexity.
This ADR narrows synchronization responsibilities so upcoming backlog tasks can be implemented with a consistent and minimal strategy.

## Decision

### Project lifecycle

- Daylite is the single source where projects are created.
- The app does not create projects in Planradar before an assignment exists.

### Assignment-first flow

- The app is always used to assign employees to projects.
- A project must be assigned to at least one employee and at least one date before any Planradar create/link step is performed.
- After this assignment exists:
  - either create a new Planradar project from the app and link it,
  - or link an already existing Planradar project from the app.

### iCal handling

- iCal calendars may contain appointments that are unrelated to app assignments.
- Unrelated iCal appointments are display-only and informative.
- Only appointments created by this app are considered assignment appointments.
- App-created iCal appointments must include metadata in the description that links back to the project.

### Legacy records

- Legacy projects/appointments may exist.
- Compatibility migration is not required for them.
- They must not crash the app; unknown/unmapped legacy data should be ignored safely.

## Consequences

- Sync logic can focus on assignment-driven behavior instead of trying to infer project ownership from legacy iCal data.
- Planradar synchronization becomes strictly gated by assignment state.
- iCal parsing can be simplified: metadata-backed appointments are actionable, all others are informational.
- Error handling should prefer safe skipping over compatibility reconstruction for old data.
