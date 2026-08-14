## 1. Harness

- [ ] 1.1 Add `@happy-dom/global-registrator` and `@testing-library/react` as devDependencies
- [ ] 1.2 Add `src/test/dom-setup.ts` that registers the DOM globally and runs Testing Library's `cleanup` after each test
- [ ] 1.3 Add `bunfig.toml` with `[test] preload` pointing at the setup file, and confirm `bun test` still passes unchanged
- [ ] 1.4 Write a throwaway spec that mounts a component with an effect and asserts the effect ran, proving the harness works end to end, then delete it

## 2. Pilot conversion

- [ ] 2.1 Convert `assignment-modal.spec.tsx` to mount the modal and query by role and label, replacing every regular expression over HTML
- [ ] 2.2 Establish whether `<dialog>` renders and behaves under happy-dom, and whether the `cancel` listener can be reached; record the answer for the ADR
- [ ] 2.3 Add the interactive cases the file could not previously express: unlocking re-enables the picker, save and delete, and closing then reopening the modal locks it again
- [ ] 2.4 Assert what an unlocked write sends, not just what it renders: after clicking the unlock checkbox, saving calls `updateAssignment` with `overrideProtection: true` and deleting calls `deleteAssignment` with `true`. Nothing covers this today, so a regression to `false` would leave the unlock button inert with the suite green
- [ ] 2.5 Reach the delete-confirm and unsaved-changes states by clicking rather than by prop

## 3. Remaining conversions

- [ ] 3.1 Convert `move-reconciliation-dialog.spec.tsx`
- [ ] 3.2 Convert `daylite-token-modal.spec.tsx`
- [ ] 3.3 Convert `employee-ical-dialog.spec.tsx`
- [ ] 3.4 Convert `timetable-cell.spec.tsx`
- [ ] 3.5 Convert `data-loading-indicator.spec.tsx`
- [ ] 3.6 Convert `settings/display-panel.spec.tsx`
- [ ] 3.7 Convert `page.spec.tsx`
- [ ] 3.8 Confirm `renderToStaticMarkup` appears nowhere in `src`

## 4. Drop the test-only props

- [ ] 4.1 Remove `showDeleteConfirm` and `showUnsavedConfirm` from `AssignmentModal`'s props. `unlocked` is already gone: it armed the protection override on every write, so it did not wait for the harness
- [ ] 4.2 Remove the matching `initialShowDeleteConfirm` and `initialShowUnsavedConfirm` inputs from `use-assignment-modal.ts`, keeping the open effect's resets
- [ ] 4.3 Confirm no component in `src` accepts a prop that only tests pass

## 5. Verification and documentation

- [ ] 5.1 Run `bun test` and confirm all tests pass, including the converted files
- [ ] 5.2 Run `bun lint` and `bunx tsc --noEmit` and fix any issues
- [ ] 5.3 Write an ADR in `docs/adr` recording the testing layers: pure logic, DOM component tests, and what is left to the Playwright layer
- [ ] 5.4 Note the component-test convention in `CLAUDE.md` so new tests land at the right level
