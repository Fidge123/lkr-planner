## 1. Theme colors

- [ ] 1.1 Add `--color-absence-vacation` (`oklch(51.1% 0.230 277)`), `--color-absence-special` (`oklch(59.1% 0.257 323)`), and `--color-absence-sick` (`oklch(65.6% 0.212 354)`) as new custom properties in `src/app.css`

## 2. Absence code classifier

- [ ] 2.1 Write failing `bun test`s for an `absenceCategoryColor(title)` function: leading-token match (case-insensitive) for `UB`, `SU`, `UU` → vacation hue at 3 intensities; `KR`, `Kro` → sick hue at 2 intensities; `FA` → special hue; unmatched → `bg-info/30`
- [ ] 2.2 Implement `absenceCategoryColor` in `src/app/types.ts`, satisfying the tests

## 3. Conflict detection

- [ ] 3.1 Write failing `bun test`s for a conflict detector: cell with absence + assignment event for the same employee/day → conflict; absence + bare event → no conflict; absence alone → no conflict
- [ ] 3.2 Implement the conflict detector, satisfying the tests

## 4. Wire into rendering

- [ ] 4.1 Update `toCellEvent()` in `src/app/types.ts` to call `absenceCategoryColor(event.title)` for `kind === "absence"` instead of the hardcoded `bg-info/30`
- [ ] 4.2 Update `src/app/components/timetable-cell.tsx` to render a red conflict indicator (ring/border + Lucide warning icon + German label, e.g. "Termin während Abwesenheit") when the conflict detector flags a cell
- [ ] 4.3 Add/update `bun test` coverage for `toCellEvent()` producing the correct color per absence code, and for the conflict indicator rendering only when flagged

## 5. Spec and verification

- [ ] 5.1 Run `bun test` and confirm all new and existing tests pass
- [ ] 5.2 Run `bun lint` and fix any issues
- [ ] 5.3 Manually verify in the running app: each absence code (`UB`, `UU`, `KR`, `Kro`, `SU`, `FA`) shows its expected color in both light and dark theme, and a day with both an absence and an appointment shows the red conflict indicator
