## Context

See proposal.md for motivation.

The frontend suite is 30 files run by `bun test`, of which 8 render components through `renderToStaticMarkup` and assert on the HTML string, usually with regular expressions over tag attributes.
The rest are pure functions and need no DOM.
Component tests already stub the backend with `mock.module("../../generated/tauri", ...)`, so nothing here touches the network or Tauri.

Two constraints shape the approach.
`bun test` is the only frontend runner and should stay that way.
React is on 19, which requires a testing library built for its `act` implementation.

`enable-agent-testing` proposes Playwright against the running app and is not yet implemented.
Nothing in this change depends on it, and nothing here forecloses it.

## Goals / Non-Goals

**Goals:**
- One harness, applied to all 8 component spec files, so the suite has a single way to test a component.
- A test reaches component state the way a user does, which is what lets the test-only props go.
- Setup lives in one preloaded file rather than in each spec.

**Non-Goals:**
- No new tests for components that have none today; this change converts and completes what exists.
- No coverage of the app shell, routing or cross-component flows, which is the Playwright layer's subject.
- No change to how the backend is stubbed.

## Decisions

### happy-dom, registered globally through a preload
`@happy-dom/global-registrator` installs `document`, `window` and the DOM constructors onto the global scope before the suite runs, wired through a new `bunfig.toml` `[test] preload`.
This is the path Bun documents for DOM tests, and one preload file keeps the setup out of the 8 spec files.

Alternative considered: jsdom.
It is the more complete implementation, but it is markedly slower to start and Bun's integration is less direct.
Given the components under test are dialogs and lists rather than exotic DOM, completeness is worth less here than start-up cost.

Alternative considered: registering the DOM per spec file.
Rejected because the registrator is global anyway, so per-file registration buys nothing and repeats itself 8 times.

### Testing Library for queries, and for the `act` boundary
`@testing-library/react` v16 supports React 19, wraps renders and events in `act`, and provides the role and label queries the specs call for.
It also owns the mount and unmount lifecycle, which is what makes "close and reopen" expressible.

Alternative considered: `react-dom/client` plus `act` directly, with hand-written queries.
Rejected: it reimplements Testing Library badly, and the query helpers are the part that removes the regular expressions over HTML.

### `fireEvent` first, `user-event` only if a test needs it
`fireEvent` covers clicking a button and changing an input, which is all the current specs need.
`@testing-library/user-event` simulates fuller interaction sequences (focus, key-by-key typing) and can be added when a test actually needs one.

### Plain assertions on DOM properties, no matcher package
`expect(saveButton.disabled).toBe(true)` states the fact directly and needs no dependency.
`@testing-library/jest-dom` would read slightly better and report better failures; it can be added later if the assertions start to feel noisy.

### Cleanup registered once, in the preload
Testing Library's `cleanup` unmounts and clears the document between tests.
Registering it as a global `afterEach` in the preload file makes isolation the default rather than something each spec has to remember.

### Convert all 8 files in this change
Leaving some files on `renderToStaticMarkup` would leave two conventions in the suite indefinitely, and the next person would have to guess which applies.
Eight files is small enough to finish, and `renderToStaticMarkup` leaves the codebase with them.

### The test-only props go last
`showDeleteConfirm`, `showUnsavedConfirm` and `unlocked` are removed only after the specs that depend on them reach those states by clicking.
Doing it in that order keeps the suite green throughout and proves the interaction path works before the prop that substituted for it disappears.

## Risks / Trade-offs

- [happy-dom's `<dialog>` support may be partial, and the modals under test are `<dialog>` elements] → The components render `<dialog open>` declaratively rather than calling `showModal()`, so the common path should work. Convert `assignment-modal.spec.tsx` first: it is the heaviest user of dialogs, so it either clears the risk early or exposes it before the other 7 files are touched.
- [The `cancel` event the modal listens for is browser behaviour that a DOM shim may not emit] → Dispatch the event directly rather than simulating Escape; if that path cannot be reached at all, leave it to the Playwright layer and say so in the ADR.
- [React 19 `act` warnings can turn async state updates into noisy or flaky tests] → Await Testing Library's `findBy*` queries for anything that resolves a mocked command, rather than asserting straight after the click.
- [Every spec file pays the DOM start-up cost, including the 22 that need none] → Accepted: one preload is simpler than two runner configurations, and the cost is start-up only. Revisit if the suite's wall time becomes noticeable.
- [Three more devDependencies on the frontend] → Accepted, and bounded: they are test-only and two of them are the de facto standard for this job.

## Migration Plan

1. Add the harness and convert `assignment-modal.spec.tsx`, which settles the dialog question.
2. Convert the remaining 7 files.
3. Add the interactive coverage each file could not previously express, including the unlock reset from `allow-fixed-appointment-override`.
4. Remove the test-only props once nothing passes them.
5. Write the ADR.

Rollback is per step: each leaves the suite green, and steps 1 and 2 can be reverted file by file.

## Open Questions

- Whether the modal's Escape-to-close path is reachable under happy-dom, or belongs to the Playwright layer. This is answerable during step 1 and changes no requirement either way.
