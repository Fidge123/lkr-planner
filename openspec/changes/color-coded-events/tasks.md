## 1. Absence category classifier

- [ ] 1.1 Write failing `bun test`s for an `absenceCategoryColor(title)` function: "krank" → `bg-error/30`, "sonderurlaub"/"fortbildung"/"schulung" → `bg-accent/30`, unmatched/vacation → `bg-info/30`, case-insensitive matching
- [ ] 1.2 Implement `absenceCategoryColor` in `src/app/types.ts`, satisfying the tests

## 2. Wire classifier into rendering

- [ ] 2.1 Update `toCellEvent()` in `src/app/types.ts` to call `absenceCategoryColor(event.title)` for `kind === "absence"` instead of the hardcoded `bg-info/30`
- [ ] 2.2 Add/update `bun test` coverage for `toCellEvent()` producing the correct color per absence category

## 3. Spec and verification

- [ ] 3.1 Run `bun test` and confirm all new and existing tests pass
- [ ] 3.2 Run `bun lint` and fix any issues
- [ ] 3.3 Manually verify in the running app: absence events with different titles (e.g. "Urlaub", "Krankheit", "Fortbildung") show visually distinct colors in the planning grid
