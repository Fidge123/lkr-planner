# Tasks

## 1. Daylite appointment API

- [ ] 1.1 Write failing tests in `src-tauri/src/integrations/daylite/appointments.rs` using a mock transport: the create body carries `subject`, `timezone: "floating"`, `local_start`, `local_end`, `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, `invitations: []`, `projects: [{"project": ...}]` and `keywords: ["lkr-planner"]`, and the created reference is read from `location`
- [ ] 1.2 Write failing tests that the search posts to `/appointments/_search?full-records=true&limit=250` with the `lkr-planner` keyword and the day's `utc_start` range, and parses `self`, `subject`, local times, `type`, `status`, `show_as`, `invitations` and linked `projects`
- [ ] 1.3 Write failing tests that a patch sends the fixed fields with `invitations: []` and no `keywords`, and that delete issues `DELETE /appointments/{id}` and treats `404` as already removed
- [ ] 1.4 Implement the DTOs and the search, create, patch and delete functions on `DayliteApiClient` with `with_daylite_tokens`, register the module in `daylite/mod.rs`, and verify `cargo test daylite::appointments` passes
- [ ] 1.5 Add the `DayliteAppointmentSync` operation to `telemetry/events.rs` and verify its snake_case serialization test passes

## 2. Desired-state planner

- [ ] 2.1 Write failing tests in `src-tauri/src/integrations/calendar/appointment_sync.rs` for building the desired appointments of a day: one per project, title `<project> (<names>)` with names deduplicated and sorted ignoring case, nickname then full name then `Unbenannter Kontakt`, earliest start and latest end across workers, the 08:00 to 16:00 window for an event without times, and bare or absence events ignored
- [ ] 2.2 Write failing tests for the action plan: create for a missing project, delete for an appointment without assignments, patch for a differing subject, time or fixed field, no action for a matching appointment, and deletion of all but the lowest-ID duplicate
- [ ] 2.3 Implement the pure planner satisfying both test sets and verify `cargo test calendar::appointment_sync` passes

## 3. Day reconciliation

- [ ] 3.1 Write failing tests for the orchestrator with stubbed calendar and Daylite access: a day whose worker calendar fails to load sends no Daylite request and records a telemetry error, a failed create for one project still applies the other projects' actions, and every Daylite failure is recorded under `DayliteAppointmentSync`
- [ ] 3.2 Implement the orchestrator: read the day from every configured primary calendar at the existing concurrency limit, resolve project names through the cached project resolution with the SUMMARY fallback, read worker names from the contact cache, search, plan and apply; verify the tests from 3.1 pass
- [ ] 3.3 Write a failing test that two reconciliations of the same date run one after the other while different dates run concurrently, then implement the per-date lock and verify the test passes

## 4. Trigger from assignment writes

- [ ] 4.1 Write failing tests that `update_assignment_core`, `move_assignment_core` and `delete_assignment_core` report the date the event had before the write, then extend their results and verify the existing `caldav::write` tests still pass
- [ ] 4.2 Schedule the background reconciliation in `create_assignment_inner`, `update_assignment_inner`, `move_assignment_inner`, `reorder_assignment_inner` and `delete_assignment_inner` for every touched date after a successful write only, and verify with tests that a failed CalDAV write schedules nothing and that the command results are unchanged
- [ ] 4.3 Start a debug build with `bun tauri dev` so the specta bindings are exported, and verify `src/generated/tauri.ts` has no diff, confirming the command contracts are unchanged

## 5. Documentation and verification

- [ ] 5.1 Write `docs/adr/0015-daylite-appointments-mirror-assignments.md` recording the whole-day desired-state reconciliation, the `lkr-planner` keyword marker, floating times and the empty invitee list, including the evaluated options with pros and cons, and verify `bun run test:docs` passes
- [ ] 5.2 Record a Daylite cassette under `tests/cassettes/` for create, search, patch and delete of an app-managed appointment against a test project, and verify the replay test passes without network access
- [ ] 5.3 Run `cargo test` and `cargo clippy` in `src-tauri`, plus `bun test` and `bun run lint`, and verify all pass
- [ ] 5.4 Manually verify in the running app: assign two workers to one project on a day, check the Daylite appointment's title, times, attributes and empty invitee list, delete one assignment and then the other, and confirm the title shrinks and the appointment is removed

## Workflow follow-up

- Run `bunx openspec validate sync-daylite-project-appointments --strict` before review.
- Archive the change with `/opsx:archive` after it is merged.
