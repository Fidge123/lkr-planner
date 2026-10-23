## Why

Component specs render with `renderToStaticMarkup`, which runs no effects and dispatches no events, so no interactive behavior can be tested.
The reset of an unlocked fixed appointment shipped untested for that reason, and `AssignmentModal` carries `showDeleteConfirm` and `showUnsavedConfirm` only so a static render can start in a state users reach by clicking.

## What Changes

- Mount components in a DOM during `bun test`, through one helper that every component spec uses.
- Cover the interactions a static render cannot reach: the unlock and its reset, the confirmation dialogs, the move reconciliation and the display toggles.
- Remove the test-only props and the `document` guards that exist only for static rendering.
- Record which test layer a new test belongs to.

## Capabilities

None: this is test tooling without observable behavior, so the change sets `skip_specs`.

## Impact

- The frontend test setup and every component spec; no production behavior changes.
- Three new dev dependencies.
