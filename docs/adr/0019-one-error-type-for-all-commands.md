# ADR 0019: One Error Type for All Commands

- Status: Accepted
- Date: 2026-10-23

## Context

Commands failed with four different shapes: a plain German string, `DayliteApiError`, `ZepError` and `StoreError`.
The frontend had to tell strings from objects, and Rust code converted between the types with `map_err` wherever one integration called another.
The frontend only ever shows `userMessage`, while telemetry needs the code, the technical message and the HTTP status.

### Evaluated Options
- One `AppError` with one code enum, used inside the integrations as well as at the command boundary
  - Pros: `?` works across integrations without conversions; one telemetry implementation; one TypeScript type.
  - Cons: The code enum mixes Daylite, ZEP and store codes.
- Keep the integration errors and convert them to a command error at the boundary
  - Pros: Each integration keeps a code enum of its own.
  - Cons: Keeps every conversion and adds one more at each command.
- Return the German message as a string from every command
  - Pros: Smallest frontend type.
  - Cons: Telemetry loses the code and the HTTP status for every failure that passes through a string.

## Decision

- Every fallible command returns `Result<T, AppError>` with `code`, `httpStatus`, `userMessage` and `technicalMessage`.
- Errors that only carry a German message convert into `AppError` with the code `COMMAND_FAILED`.
- Frontend services turn a command result into data or a thrown `Error` carrying `userMessage` through `callCommand`, which also maps IPC rejections to a German fallback.
- Infallible commands keep returning their value directly.

## Consequences

- Telemetry records the original code of a store or keychain failure even when it surfaces through another integration's command.
- The frontend has a single error shape to handle.
