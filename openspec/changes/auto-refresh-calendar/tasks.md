# Tasks

## 1. Refresh entry point in `usePlanningAssignments`

- [ ] 1.1 Write failing `bun test`s in `src/app/hooks/use-planning-assignments.spec.ts` for the fetch time recorded per cached week and the derived `lastLoadedAt`: set on a successful fetch, kept from the cache entry on a cache hit, set to the completion time on a failed load
- [ ] 1.2 Record the fetch time per cached week and expose `lastLoadedAt` from `src/app/hooks/use-planning-assignments.ts`, and verify the tests from 1.1 pass
- [ ] 1.3 Write failing `bun test`s for `refreshAssignments()`: it discards every cached week, keeps the displayed events and per-employee errors when the fetch fails while setting the German `errorMessage`, and clears `errorMessage` on success
- [ ] 1.4 Implement `refreshAssignments()` in `use-planning-assignments.ts` and verify the tests from 1.3 pass and the existing `reloadAssignments()` tests still pass unchanged

## 2. Scheduling hook

- [ ] 2.1 Write failing `bun test`s in `src/app/hooks/use-calendar-auto-refresh.spec.ts` for the scheduling logic with injected clock, timer and visibility: refresh due 5 minutes after `lastLoadedAt`, re-armed when `lastLoadedAt` changes, skipped while hidden, immediate on visibility or focus when due, no refresh on focus when not due
- [ ] 2.2 Implement `src/app/hooks/use-calendar-auto-refresh.ts` with the 5 minute interval constant, `visibilitychange` and window `focus` listeners, and cleanup on unmount, and verify the tests from 2.1 pass

## 3. Wiring and decision record

- [ ] 3.1 Call `useCalendarAutoRefresh` in `src/app.tsx` with `lastLoadedAt` and `refreshAssignments` from `planningAssignmentsState`, and verify `bun run build` type-checks
- [ ] 3.2 Write ADR `docs/adr/0015-periodic-calendar-refresh.md` recording client-side polling over push or backend polling, and verify `bun run test:docs` passes

## 4. Verification

- [ ] 4.1 Run `bun test` and `bun run lint` and confirm both pass without changes left by the linter
- [ ] 4.2 Verify in the running app with the interval temporarily shortened: an appointment changed directly in CalDAV appears without user action, a cached week is fetched again after a refresh, and disconnecting the network keeps the grid filled while the German error with "Erneut laden" appears
