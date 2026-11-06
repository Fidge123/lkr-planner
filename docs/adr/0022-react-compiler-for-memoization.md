# ADR 0022: React Compiler for Memoization

- Status: Accepted
- Date: 2026-10-23

## Context

The planning grid re-rendered every card on each highlight toggle and drag move.
With 25 employees and three cards per day, a profiling build in Chromium spent 22 to 33 ms of React work per highlight toggle and 10 to 12 ms per pointer move while dragging.
Rows filtered their events per cell on every render and passed new callbacks to every cell, so no card kept its props.

### Evaluated Options
- React Compiler through Babel
  - Pros: Memoizes components and hooks without dependency lists; with stable props it cut a highlight toggle to 9 to 10 ms and a drag move to about 4.5 ms.
  - Cons: Three build dependencies, 11.5 kB more gzipped bundle, and specs run the uncompiled code.
- Manual `memo`, `useMemo` and `useCallback`
  - Pros: No build change, and specs run the shipped code.
  - Cons: Every prop path down to the cards needs hand-kept dependency lists that drift silently.
- The Rust port of the compiler in `@vitejs/plugin-react`
  - Pros: No Babel step.
  - Cons: Marked experimental.

## Decision

- The build compiles components and hooks with the React Compiler, and code it cannot compile stays as written.
- Components read the date through `useToday`, and date helpers take `today` as an argument instead of reading the clock.
- Props stay stable: a callback that needs a value from a loop takes it as an argument, and values derived for a loop are computed before it.
- Manual memoization is added only where a measurement shows the compiler leaves a value unmemoized.

## Consequences

- A highlight toggle re-renders only the cards it changes, and a drag move costs about half as much as before.
- Week navigation still mounts every card of the new week, and measuring drop targets during a drag remains its main cost.
- Hooks that keep the latest value in a ref during render, and code with `try`/`finally`, stay uncompiled.
- A behavior only the compiled build shows needs a check in the built app, since specs run uncompiled.
