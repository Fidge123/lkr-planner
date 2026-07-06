## 1. Theme colors

- [ ] 1.1 Add `--color-absence-vacation` (`oklch(51.1% 0.230 277)`), `--color-absence-special` (`oklch(59.1% 0.257 323)`), and `--color-absence-sick` (`oklch(65.6% 0.212 354)`) as new custom properties in `src/app.css`

## 2. Conflict detection

- [ ] 2.1 Write failing `bun test`s for a conflict detector: cell with absence + assignment event for the same employee/day → conflict; absence + bare event → conflict; absence alone → no conflict
- [ ] 2.2 Implement the conflict detector, satisfying the tests

## 3. Wire into rendering

- [ ] 3.1 Update `toCellEvent()` in `src/app/types.ts` to call `absenceCategoryColor(event.title)` for `kind === "absence"` instead of the hardcoded `bg-info/30`
- [ ] 3.2 Update `src/app/components/timetable-cell.tsx` to render a red conflict indicator (ring/border + Lucide warning icon) when the conflict detector flags a cell
- [ ] 3.3 Add/update `bun test` coverage for `toCellEvent()` producing the correct color per absence code, and for the conflict indicator rendering only when flagged
