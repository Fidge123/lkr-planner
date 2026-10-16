## Why

Component tests render with `renderToStaticMarkup`, which produces the first render and nothing else: effects never run, state never changes, and no event can be dispatched.
Every interactive behavior is therefore untestable, and the gap is not theoretical.
`allow-fixed-appointment-override` specifies that reopening the modal drops the unlock, and that scenario shipped without a test because there is no way to express it.
The modal is permanently mounted per employee row, so the reset is the only thing standing between an unlock and its leaking onto the next assignment opened in that row.

The workaround has a second cost.
`AssignmentModal` accepts `showDeleteConfirm` and `showUnsavedConfirm` purely so a static render can start in a state the user reaches by clicking; no production caller passes either of them.
It accepted an `unlocked` prop on the same grounds until that one was removed separately, for arming the protection override rather than opening a dialog.
Each new interactive state adds another prop to the public interface of a component that does not need it.

## What Changes

- Add a DOM environment to `bun test` so components can be mounted, interacted with and asserted against as they behave, not just as they first render.
- Add a React testing library so tests query rendered output by role and label instead of matching HTML with regular expressions.
- Convert the eight `renderToStaticMarkup` spec files to the new harness, and cover the interactive behavior each one currently cannot reach.
- Remove the test-only initial-state props from `AssignmentModal` once its states are reachable by interaction.
- Record the testing layers as an ADR, so it is clear which level a new test belongs at.

## Capabilities

### New Capabilities

- `component-interaction-testing`: a DOM-backed component test harness in `bun test` that mounts components, dispatches user events, runs effects and asserts on the accessible tree.

### Modified Capabilities

None.
Removing the test-only props changes no requirement in `assignment-modal-crud`: they are invisible to users and no production caller passes them.

## Impact

- `package.json`: new devDependencies for the DOM environment and the React testing library; no change to the `test` script.
- `bunfig.toml`: new file preloading the DOM registrator before the test run.
- `src/app/**/*.spec.tsx`: the eight static-markup spec files move to the new harness.
- `src/app/components/assignment-modal.tsx` and `src/app/hooks/use-assignment-modal.ts`: the `showDeleteConfirm` and `showUnsavedConfirm` props and their hook inputs go away.
- `docs/adr`: a new ADR on the testing layers.
- No change to the Rust backend, to `cargo test`, or to the pure-logic specs that need no DOM.

## Dependencies

None.
This change is independent of `enable-agent-testing`, which proposes Playwright against the running app.
The two sit at different levels and neither blocks the other; the ADR in this change records where the boundary between them lies.
