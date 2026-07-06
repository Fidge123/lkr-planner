# Refactoring Plan: Readability, File Granularity, Wrappers, and Parameter Lists

Status: Proposed
Date: 2026-07-06

## Goal

A codebase that can be understood quickly, where a change only needs to touch the files relevant to it, and where parallel changes do not collide in shared hotspot files.
This plan targets four observed problems: redundant comments, oversized files with section-marker comments, thin wrappers with little logic, and functions with long parameter lists.
Every step is a behavior-preserving refactor verified by the existing test suite (`bun test`, `cargo test`, `bun lint`).

## Findings

### 1. Comments

Most comments in this codebase are good constraint comments and must stay.
Examples worth keeping: the Daylite bare `{}` empty-response quirk (`daylite/shared.rs`), the token rotation lock rationale (`daylite/shared.rs`), the CalDAV href-vs-origin resolution rule (`calendar/caldav.rs`), and the BOM/zero-width-space stripping (`calendar/events.rs`).

Three categories should be removed or made redundant by naming:

- Narration comments that restate the next line.
  Examples: `// Try the local Daylite cache first.` and `// Placeholder: project could not be resolved.` in `calendar/events.rs`, `// Focus the filter so the user can start typing immediately.` in `assignment-modal.tsx`.
- History comments that talk to a past reviewer instead of the next reader.
  Example: `daylite/projects.rs` in `fetch_project_by_reference` explains what "the previous code" did wrong.
  The constraint (the lock persists the rotated token) belongs on `with_token_refresh_lock`, where it already exists.
- Duplicated default documentation.
  The defaults for `hideNonPlannableEmployees` and `showWeekend` are documented three times: in `local_store.rs`, in `display-settings.ts`, and again as inline comments in `app.tsx`.
  The Rust struct is the source of truth; the frontend copies should go.
- Phase narration that should become function names.
  `load_week_events` in `calendar/commands.rs` uses `// First pass: ...` and `// Second pass: ...` comments over ~170 lines.
  Extracting named functions (`fetch_all_employee_calendars`, `fetch_uncached_projects`, `assemble_employee_events`) removes the comments and shrinks the function.

### 2. Large files with section comments

Files using `// ── Section ──` markers are self-identified split candidates; the marker names are the new file names.

| File | Lines | Sections / contents | Split |
|---|---|---|---|
| `daylite/projects.rs` | 1444 | ~450 prod + ~1000 test lines | Move tests to `projects/tests.rs`; extract shared test support (see finding 5) |
| `calendar/events.rs` | 721 | classification, resolution, ordering, absence mapping | `calendar/events/` with `classify.rs`, `resolve.rs`, `absences.rs` |
| `local_store.rs` | 595 | types, persistence commands, tests | `local_store/types.rs` + `local_store/persistence.rs` |
| `settings-dialog.tsx` | 519 | dialog shell + 3 panel components | `components/settings/` with one file per panel |
| `assignment-modal.tsx` | 502 | modal + 2 confirm dialogs + list + 4 pure helpers | Extract confirm dialogs, `ProjectResultList`, and pure helpers |
| `calendar/caldav.rs` | 498 | fetch, helpers, write cores | `caldav/report.rs` (fetch + parse) + `caldav/write.rs` |
| `daylite/contacts/api.rs` | 515 | already modular, mostly tests | Test support extraction only |
| `employee-ical-dialog.tsx` | 370 | dialog + `CalendarSection` | Extract `CalendarSection` |

Section markers inside spec files (`assignment-modal.spec.tsx`, `next-day-quick-add.spec.ts`) reference backlog task IDs and act as test documentation; they stay.

### 3. Thin wrappers

- Every function in `src/services/*.ts` follows the same shape: call the generated command, check `result.status`, throw a `new Error` with a German fallback.
  `readZepErrorMessage` (`zep.ts`) and `readDayliteApiErrorMessage` (`daylite-service-helpers.ts`) are near-identical shape checks.
  ADR 0002 mandates the service facade, so the layer stays, but one generic helper `unwrapCommandResult(result, fallbackMessage)` collapses each wrapper to one line and deletes both error readers.
- `display-settings.ts` has four functions (`loadHideNonPlannableEmployees`, `saveHideNonPlannableEmployees`, `loadShowWeekend`, `saveShowWeekend`) that each load the whole store.
  Replace with `loadDisplaySettings(): Promise<DisplaySettings>` and `saveDisplaySettings(patch: Partial<DisplaySettings>)`.
  This also removes duplicate load calls in `app.tsx` and `DisplaySettingsPanel`, and the merge-comment in both save functions becomes unnecessary.
- `PlanningGrid` in `page.tsx` only resolves optional props and delegates to `PlanningGridTable`.
  Merge them; tests that need injected state can pass the optional props directly.
- The three Daylite project commands (`daylite_list_projects`, `daylite_search_projects`, `daylite_query_overdue_projects`) repeat the same five lines: load store, build client, run core under the token lock, save store.
  The store save is a round-trip of unchanged data and needs a second look while extracting.
  Extract `run_daylite_command(app, |client, tokens| ...)` in `daylite/shared.rs`.
- The three calendar write commands in `calendar/commands.rs` each rebuild the reqwest client, reload credentials, and reload the store.
  Extract a `CaldavContext::load(app)` constructor.

### 4. Long parameter lists

- `create_assignment_core` (8 params) and `update_assignment_core` (10 params) in `calendar/caldav.rs` carry `#[allow(clippy::too_many_arguments)]`.
  Introduce two structs: `CaldavSession { client, username, password, base_url, absence_urls }` and `AssignmentWrite { uid, date, project_ref, project_name }`.
  Both clippy allows disappear, and `delete_assignment_core` (6 params) becomes `(session, href)`.
- `send_authenticated_json` / `send_authenticated_request` in `daylite/auth_flow.rs` take 6 positional params.
  `DayliteHttpRequest` already exists in `client.rs`; accept it (or a builder) instead of loose method/path/query/body params.
- The Tauri command `update_assignment(app, href, uid, date, project_ref, project_name)` leaks the positional list into the generated TypeScript (`commands.updateAssignment(a, b, c, d, e)`).
  Change the command to take a single input struct, which specta generates as a named-field object on the frontend.
- `TimetableRow` takes 9 props, two of which (`onRetry`, `onReloadAssignments`) receive the same `reloadAssignments` function from `page.tsx`.
  Merge those two into one callback and group `weekDays` + `holidayDates` into a `week` prop.
- `EmployeeIcalDialog` takes 8 props, four of which (`zepCalendars`, `isLoadingCalendars`, `calendarsError`, `onReloadCalendars`) are one concern.
  Move that state into a `useZepCalendars` hook consumed by the dialog, which also removes three `useState` calls and a loader from `app.tsx`.

### 5. Test boilerplate (main driver of file size)

- `MockTransport` is copy-pasted into three files: `daylite/projects.rs`, `daylite/contacts/api.rs`, and `daylite/auth_flow.rs`.
- The `DayliteTokenState { access_token: "at", ... }` literal appears ~20 times in `projects.rs` alone.
- `DayliteSearchInput` is constructed with all six fields spelled out in every test, even when only `search_term` matters.

Fix: a `#[cfg(test)] mod test_support` under `daylite/` providing `MockTransport`, `mock_response`, and a `token_state()` fixture, plus `#[derive(Default)]` on `DayliteSearchInput` so tests write `DayliteSearchInput { search_term: "Nord".into(), ..Default::default() }`.
This alone removes several hundred lines and is the reason `projects.rs` is the largest file in the repo.

## Execution plan

Each phase is an independent, mergeable unit that leaves the build green.
Comment cleanup (finding 1) is folded into whichever phase touches the file, so no file is churned twice; a final sweep catches the rest.

### Phase 1: Rust test support (no production code changes)

1. Add `daylite/test_support.rs` with `MockTransport`, `mock_response`, and token fixtures; delete the three copies.
2. Derive `Default` for `DayliteSearchInput` and collapse test literals.
3. Move the test module of `projects.rs` to `daylite/projects/tests.rs`.

### Phase 2: Parameter objects in Rust

1. Introduce `CaldavSession` and `AssignmentWrite` in `calendar/caldav.rs`; remove both `#[allow(clippy::too_many_arguments)]`.
2. Change `send_authenticated_json` / `send_authenticated_request` to take a request struct.
3. Change `update_assignment` and `create_assignment` commands to single input structs and regenerate bindings (touches `assignment-modal.tsx` call sites).

### Phase 3: Command boilerplate helpers in Rust

1. Extract `run_daylite_command` in `daylite/shared.rs`; investigate and remove the no-op store save if confirmed unnecessary.
2. Extract `CaldavContext::load(app)` for the calendar write commands.
3. Break `load_week_events` into the three named phase functions.

### Phase 4: File splits

1. `calendar/events.rs` into `events/classify.rs`, `events/resolve.rs`, `events/absences.rs` with co-located tests.
2. `calendar/caldav.rs` into `caldav/report.rs` and `caldav/write.rs`.
3. `local_store.rs` into `local_store/types.rs` and `local_store/persistence.rs`.
4. `settings-dialog.tsx` into `components/settings/` (dialog shell, `daylite-panel.tsx`, `zep-panel.tsx`, `display-panel.tsx`, shared status alert).
5. `assignment-modal.tsx`: extract `unsaved-changes-dialog.tsx`, `delete-confirm-dialog.tsx`, `project-result-list.tsx`, and move the pure helpers to `assignment-modal-logic.ts`.
6. `employee-ical-dialog.tsx`: extract `calendar-section.tsx`.

### Phase 5: Frontend service and prop cleanups

1. Add `unwrapCommandResult` to a shared service helper; collapse the wrappers in `zep.ts`, `daylite-*.ts`, and `health.ts`; delete `readZepErrorMessage` and `readDayliteApiErrorMessage`.
2. Replace the four `display-settings.ts` functions with `loadDisplaySettings` / `saveDisplaySettings`.
3. Merge `PlanningGrid` and `PlanningGridTable`.
4. Add `useZepCalendars` and shrink `EmployeeIcalDialog` and `app.tsx`.
5. Merge the duplicate reload callbacks and group props on `TimetableRow`.

### Phase 6: Final comment sweep

Remove remaining narration and history comments repo-wide using the rule: a comment stays only if it states a constraint the code cannot express (API quirk, ordering requirement, safety guard rationale).

## Non-goals

- No behavior changes, no new features, and no changes to German user-facing texts.
- The `lib.rs` command registration list stays a single `collect_commands!` block (macro requirement); keeping it grouped by integration is enough to limit merge conflicts.
- `src/generated/tauri.ts` is generated and exempt from all of the above.
- Section markers in spec files that map tests to backlog IDs stay.

## Verification per phase

Run `bun lint`, `bun test`, and `cargo test` (including VCR replay tests) after each phase.
Phases 2 and 3 regenerate `src/generated/tauri.ts` via the `regenerate_generated_bindings` test.
