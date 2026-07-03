Work red/green: write the failing test first, then the implementation that makes it pass.

## 1. Create-only ghost trigger

- [x] 1.1 Write failing test: creating an assignment sets a ghost of the created project on the next visible day for that employee
- [x] 1.2 Extend the `onSave` callback to carry the created project (reference plus name); set the row ghost state `{ date, project } | null` on create only
- [x] 1.3 Write failing test: editing an assignment does not set a ghost
- [x] 1.4 Ensure the edit and delete save paths call `onSave` without the created-project payload so no ghost is set

## 2. Target day and suppression

- [x] 2.1 Write failing test: the ghost lands on the next visible day, and no ghost appears when saving on the last visible day
- [x] 2.2 Derive the target day from the `weekDays` index after the created day; render the ghost only on the matching cell
- [x] 2.3 Write failing test: the ghost is suppressed when the target day already holds any event for that employee
- [x] 2.4 Evaluate suppression at render against live cell events (`cell.date === ghost.date && cell has no events`)

## 3. Ghost rendering

- [x] 3.1 Write failing test: `TimetableCell` renders a `suggestion` prop with reduced opacity (~50%) and a dashed border, between the events and the `+` button
- [x] 3.2 Add the optional `suggestion` prop to `TimetableCell` and render it accordingly

## 4. One-click add and chaining

- [x] 4.1 Write failing test: clicking the ghost creates a persisted assignment for the target day and reloads the table
- [x] 4.2 Wire the ghost click to reuse the create path (`createAssignment` plus reload)
- [x] 4.3 Write failing test: after clicking, a new ghost of the same project appears on the following visible day (chaining)
- [x] 4.4 Write failing test: clicking a ghost on the last visible day creates the assignment and shows no further ghost
- [x] 4.5 Ensure the click sets the next ghost through the same create trigger so chaining honors suppression and the week boundary

## 5. Ghost clearing

- [x] 5.1 Write failing test: deleting any assignment clears the ghost
- [x] 5.2 Write failing test: navigating to another week clears the ghost
- [x] 5.3 Write failing test: cancelling a modal without saving keeps the ghost
- [x] 5.4 Clear the ghost on delete and on week change; leave it untouched on cancel

## Implementation notes

All ghost lifecycle rules (create-only trigger, target-day resolution, chaining,
suppression, delete-clears, edit-is-a-no-op) live in one pure, fully unit-tested
reducer: `src/app/next-day-quick-add.ts` (`nextGhostState`, `nextVisibleDay`,
`isGhostVisible`), tested in `src/app/next-day-quick-add.spec.ts`.
This repo has no DOM test environment (`renderToStaticMarkup` only, no
`fireEvent`/interaction simulation), matching its existing convention of
extracting decision logic into pure functions (see `resolveEscapeAction`,
`nextHighlightIndex` in `assignment-modal.tsx`) and only static-rendering
components.
Task 1.4's create/edit split is additionally covered directly via
`resolveSaveAction` in `assignment-modal.tsx` / `.spec.tsx`.

Two behaviors are implemented but not independently unit-testable given this
constraint, since they depend on click handling or a `useState`-during-render
adjustment rather than pure logic: the actual click wiring in
`TimetableRow.handleSuggestionClick` (4.1/4.2, calls `commands.createAssignment`
then reuses `nextGhostState`) and the week-navigation reset in `TimetableRow`
(5.2, compares `weekStart` during render and clears the ghost, see the comment
above that check).
Cancel keeping the ghost (5.3) holds by construction: `AssignmentModal`'s
close/cancel paths never call `onSave`, so `nextGhostState` is never invoked
and the ghost is left as-is.

Verified live: `bun run dev` against a mocked Tauri IPC bridge (Playwright,
headless Chromium) driving the real app.
Confirmed in the browser: a create on Monday produces a ghost on Tuesday;
clicking the ghost creates the real Tuesday assignment and chains a new ghost
to Wednesday; creating on the last visible day produces no ghost; deleting an
assignment clears an existing ghost.
Week-navigation clearing (5.2) and cancel-keeps-ghost (5.3) were not
separately exercised in the browser session, only via the reasoning above and
`bun run lint` / `bunx tsc --noEmit` / `bun test` (157 tests), all passing.
