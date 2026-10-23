# Tasks

## 1. Refresh entry point in `usePlanningAssignments`

- [x] 1.1 Write failing `bun test`s in `src/app/hooks/use-planning-assignments.spec.ts` for the fetch time recorded per cached week and the derived `lastLoadedAt`: set on a successful fetch, kept from the cache entry on a cache hit, set to the completion time on a failed load
- [x] 1.2 Record the fetch time per cached week and expose `lastLoadedAt` from `src/app/hooks/use-planning-assignments.ts`, and verify the tests from 1.1 pass
- [x] 1.3 Write failing `bun test`s for `refreshAssignments()`: it discards every cached week, keeps the displayed events and per-employee errors when the fetch fails, and clears `errorMessage` on success
- [x] 1.4 Implement `refreshAssignments()` in `use-planning-assignments.ts` and verify the tests from 1.3 pass and the existing `reloadAssignments()` tests still pass unchanged
- [x] 1.5 Write failing `bun test`s for the silent retry: a failed automatic refresh leaves `errorMessage` and `lastLoadedAt` unchanged and retries once after 30 seconds, a failed retry sets the German `errorMessage` and `lastLoadedAt`, and any other completed load of the displayed week drops the pending retry
- [x] 1.6 Implement the retry in `use-planning-assignments.ts` with an injectable timer, clear the pending retry on unmount, and verify the tests from 1.5 pass

## 2. Scheduling hook

- [x] 2.1 Write failing `bun test`s in `src/app/hooks/use-calendar-auto-refresh.spec.ts` for the scheduling logic with injected clock, timer, visibility and online state: refresh due 5 minutes after `lastLoadedAt`, re-armed when `lastLoadedAt` changes, skipped while hidden or offline, a due check 5 seconds after visibility, focus or `online`, no refresh on such an event when not due
- [x] 2.2 Implement `src/app/hooks/use-calendar-auto-refresh.ts` with the interval and settle delay constants, `visibilitychange`, window `focus` and window `online` listeners, and cleanup of listeners and timers on unmount, and verify the tests from 2.1 pass

## 3. Wiring and decision record

- [x] 3.1 Call `useCalendarAutoRefresh` in `src/app.tsx` with `lastLoadedAt` and `refreshAssignments` from `planningAssignmentsState`, and verify `bun run build` type-checks
- [x] 3.2 Write ADR `docs/adr/0018-periodic-calendar-refresh.md` recording client-side polling over push or backend polling and the offline, settle delay and retry rules that protect Daylite token rotation after wake, and verify `bun run test:docs` passes

## 4. Verification

- [x] 4.1 Run `bun test` and `bun run lint` and confirm both pass without changes left by the linter
- [ ] 4.2 Verify in the running app with the interval temporarily shortened: an appointment changed directly in CalDAV appears without user action, a cached week is fetched again after a refresh, disconnecting the network skips refreshes without an error, reconnecting refreshes 5 seconds later, and a failing server keeps the grid filled and shows the German error with "Erneut laden" only after the silent retry also failed
