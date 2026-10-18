# Tasks

## 1. Daylite appointment API

- [x] 1.1 Write failing tests in `src-tauri/src/integrations/daylite/appointments.rs` using a mock transport: the create body carries `subject`, `timezone: "floating"`, `local_start`, `local_end`, `type: "appointment"`, `status: "confirmed"`, `show_as: "free"`, `invitations: []`, `projects: [{"project": ...}]`, `contacts: [{"contact": <worker>}]` and `keywords: ["lkr-planner"]`, and the created reference is read from `location`
- [x] 1.2 Write failing tests that the search posts to `/appointments/_search?full-records=true&limit=250` with the `lkr-planner` keyword and the day's `utc_start` range, and parses `self`, `subject`, local times, `type`, `status`, `show_as`, `invitations`, linked `projects` and linked `contacts`
- [x] 1.3 Write failing tests that an update sends the fixed fields with `invitations: []`, no `keywords`, and new contacts only through `contacts.add`, that an unlink sends only `contacts.remove` with the worker, and that delete issues `DELETE /appointments/{id}` and treats `404` as already removed
- [x] 1.4 Implement the appointment record, the `AppointmentStore` trait and the search, create, update, unlink and delete requests with `with_daylite_tokens`, register the module in `daylite/mod.rs`, and verify `cargo test daylite::appointments` passes
- [x] 1.5 Add the `DayliteAppointmentSync` operation to `telemetry/events.rs` and verify its snake_case serialization test passes

## 2. Edited-worker planner

- [x] 2.1 Write failing tests in `src-tauri/src/integrations/calendar/appointment_sync.rs` for the appointment fields: the title `Einsatz <dd.MM.yyyy>` and the times 08:00 to 16:00, and for the worker's projects of the day, ignoring bare events and other days
- [x] 2.2 Write failing tests for the worker's action plan: create for a project without an appointment, link the worker to an existing appointment, restore a differing fixed field, no action for a matching linked appointment, unlink the worker from a project it left, delete an appointment whose last planning contact is the worker, leave other workers' appointments alone, and let the lowest-ID duplicate take over the others' contacts
- [x] 2.3 Implement the pure planner satisfying the tests and verify `cargo test calendar::appointment_sync` passes

## 3. Worker day synchronization

- [x] 3.1 Write failing tests for the orchestrator with a stubbed Daylite store: a worker whose calendar fails to load sends no Daylite request and records a telemetry error, a failed create still applies the worker's other actions, duplicates survive a failed takeover of their contacts, and every Daylite failure is recorded under `DayliteAppointmentSync`
- [x] 3.2 Implement the orchestrator: read the day from the edited worker's primary calendar, search, plan and apply; verify the tests from 3.1 pass
- [x] 3.3 Write a failing test that two synchronizations of the same date run one after the other while different dates run concurrently, then implement the per-date lock and verify the test passes

## 4. Trigger from assignment writes

- [x] 4.1 Write failing tests that `update_assignment_core`, `move_assignment_core` and `delete_assignment_core` report the date the event had before the write, then extend their results and verify the existing `caldav::write` tests still pass
- [x] 4.2 Write a failing test that an event's href maps to the employee whose configured primary calendar holds it, also when the configured URL differs in host case, default port or path encoding, then implement the mapping on the shared `lies_in_calendar` helper that the absence guard also uses
- [x] 4.3 Schedule the background synchronization in `create_assignment_inner`, `update_assignment_inner`, `move_assignment_inner` and `delete_assignment_inner` for every touched worker day after a successful write only, and verify with tests that a failed CalDAV write schedules nothing, that the command results are unchanged, that a reorder schedules nothing, and that a write's worker days run one after the other with a move's target worker before its source worker

## 5. Failure registry and retry command

- [x] 5.1 Write failing tests for the failed worker day registry: a failed synchronization adds its worker day, a successful one removes it, a worker day whose calendar could not be read counts as failed, a date stays listed while another worker on it still fails, and every change emits `daylite-appointment-sync-failures` with the sorted ISO dates while the registry lock is held; implement it in `appointment_sync.rs` and verify the tests pass
- [x] 5.2 Write failing tests for `retry_daylite_appointment_sync`: it synchronizes every failed worker day, at most two at once, keeps only the worker days that fail again, resolves after all of them finished, and waits for the per-date lock of a date a write is synchronizing; implement the command, register it in `src-tauri/src/lib.rs`, and verify the tests pass
- [x] 5.3 Export the specta bindings with the `regenerate_generated_bindings` test and verify the only diff in `src/generated/tauri.ts` is the new `retryDayliteAppointmentSync` command

## 6. Failure alert

- [x] 6.1 Write failing tests for the shared Tauri event subscription in `src/app/tauri-event.spec.ts`: it listens to the named event, passes each payload on and unsubscribes through the pending promise on unmount; implement `use-appointment-sync-failures.ts` on top of it with an `isRetrying` flag that disables the button while the retry command runs, and verify `bun test` passes
- [x] 6.2 Write failing render tests for the alert: hidden without failed days, the German message with dates formatted `dd.MM.yyyy` in ascending order, the `Erneut versuchen` button, and the disabled `Wird wiederholt...` state while retrying; render it in `src/app/page.tsx` above the planning grid and verify `bun test` passes

## 7. Documentation and verification

- [x] 7.1 Write `docs/adr/0017-daylite-appointments-mirror-assignments.md` recording the edited-worker synchronization, the linked contacts, the `lkr-planner` keyword marker, the fixed floating times, the empty invitee list, and the failure registry with its event and retry command, including the evaluated options with pros and cons, and verify `bun run test:docs` passes
- [ ] 7.2 Record a Daylite cassette under `tests/cassettes/` for create, search, update, unlink and delete of an app-managed appointment against a test project and contact, and verify the replay test passes without network access
- [x] 7.3 Run `cargo test` and `cargo clippy` in `src-tauri`, plus `bun test` and `bun run lint`, and verify all pass
- [ ] 7.4 Manually verify in the running app: assign two workers to one project on a day, check the Daylite appointment's title `Einsatz <date>`, the 08:00 to 16:00 times, its attributes, both linked contacts and the empty invitee list, delete one assignment and then the other, and confirm the first worker is unlinked and the appointment is then removed; then block Daylite access, create an assignment, confirm the alert names the day, restore access, click `Erneut versuchen`, and confirm the alert disappears and the appointment exists

## Workflow follow-up

- Run `bunx openspec validate sync-daylite-project-appointments --strict` before review.
- Archive the change with `/opsx:archive` after it is merged.
