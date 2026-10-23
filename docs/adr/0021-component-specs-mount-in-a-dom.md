# ADR 0021: Component Specs Mount in a DOM

- Status: Accepted
- Date: 2026-10-23

## Context

Component specs rendered static markup with `react-dom/server`, which runs no effects and handles no events.
To test a dialog state, components grew props that only tests set, and code guarded `document` for the server renderer.
Interactions such as unlocking a fixed appointment stayed untested.

### Evaluated Options
- happy-dom with Testing Library in `bun test`
  - Pros: Specs click and type in the existing runner without a browser, and the suite stays fast.
  - Cons: Two dev dependencies, and happy-dom computes no layout.
- Playwright tests against the built frontend
  - Pros: A real browser engine.
  - Cons: A second runner, slower feedback, and the Tauri backend would need its own mock layer.
- Keep static rendering
  - Pros: No dependencies.
  - Cons: Test-only props and guards keep growing, and interactions stay untested.

## Decision

- Pure logic is tested as plain functions, services against mocked commands, hooks and the week cache against a query client, and components mounted in happy-dom.
- Component specs mount through `renderComponent`, which wraps them in a fresh query client that a spec can seed.
- A spec reaches a state by clicking or typing, never through props that only tests set.
- The test preload mocks the Tauri IPC: events reach `listen` through `emit`, and a command no spec mocked fails.

## Consequences

- Specs assert on markup or query by role and text, and inline styles serialize as the DOM writes them.
- Behavior that depends on layout, such as drag measurement and the swipe gesture, stays covered by logic and hook specs.
- `mockCommands` stays process-wide, so each spec mocks the commands it needs.
