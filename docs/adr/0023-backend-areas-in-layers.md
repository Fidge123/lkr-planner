# ADR 0023: Backend Areas in Layers

- Status: Accepted
- Date: 2026-10-23

## Context

Backend areas used each other in both directions: Daylite and ZEP each called the other for employee calendars, and telemetry and the local store each used the other for the telemetry settings.
A cycle ties two areas together so that neither can change or be tested alone, and nothing stopped new cycles from appearing.

### Evaluated Options
- Ranked layers checked by a test
  - Pros: Fixes the direction of every dependency without moving files, and the test names each violation.
  - Cons: A custom test parses the sources, and each new area has to be assigned a layer.
- Splitting the backend into `planning`, `services` and `clients` directories
  - Pros: The directory states the role of each file.
  - Cons: Moves most of the backend and splits files that mix roles, while the cycles came from two commands.
- `cargo modules` or a similar tool in CI
  - Pros: No checking code to maintain.
  - Cons: Finds cycles but not the direction between layers, and adds an install step to CI.

## Decision

- Areas form layers, from the bottom: `error`, `http` and `secret_manager`; `telemetry`; `local_store`; `daylite`, `holidays` and `zep`; `calendar`.
- An area uses only areas of lower layers, so areas on one layer, like Daylite and ZEP, never use each other.
- Work that combines areas of one layer lives in an area above them, like the employee calendar commands in `calendar`.
- `src-tauri/src/architecture.rs` checks the rule on every `cargo test` and ignores test code.

## Consequences

- A new area fails the check until it is assigned a layer.
- Cycles between files inside one area are not checked, and two remain inside Daylite.
