## 1. Theme colors

- [ ] 1.1 Add `--color-absence-vacation` (`oklch(51.1% 0.230 277)`), `--color-absence-special` (`oklch(59.1% 0.257 323)`), and `--color-absence-sick` (`oklch(65.6% 0.212 354)`) as new custom properties in `src/app.css`

## 2. Conflict detection

- [ ] 2.1 Write failing `bun test`s for a conflict detector: cell with absence + assignment event for the same employee/day → conflict; absence + bare event → conflict; absence alone → no conflict
- [ ] 2.2 Implement the conflict detector, satisfying the tests

## 3. Wire into rendering

- [ ] 3.1 Update `toCellEvent()` in `src/app/types.ts` to call `absenceCategoryColor(event.title)` for `kind === "absence"` instead of the hardcoded `bg-info/30`
- [ ] 3.2 Update `src/app/components/timetable-cell.tsx` to render a red conflict indicator (ring/border + Lucide warning icon) when the conflict detector flags a cell
- [ ] 3.3 Add/update `bun test` coverage for `toCellEvent()` producing the correct color per absence code, and for the conflict indicator rendering only when flagged

## 4. Daylite category colors for assignment events

- [ ] 4.1 Write failing `cargo test`s for a categories fetch core: sends `GET /categories` with `entity=project`, parses `name` and nullable `hex_colour`, and keeps inactive categories
- [ ] 4.2 Implement the categories fetch in `src-tauri/src/integrations/daylite`, satisfying the tests
- [ ] 4.3 Write failing `cargo test`s for the calendar pipeline: `resolve_event` sets `category_color` on `CalendarCellEvent` from both the cache and the API fallback path, and leaves it unset for bare events, absence events, and unresolved projects
- [ ] 4.4 Thread the project category through `DayliteProjectCacheEntry`, `fetch_project_by_reference`, and `CalendarCellEvent`, regenerate the TypeScript bindings, satisfying the tests
- [ ] 4.5 Write failing `bun test`s for assignment coloring: event with `category_color` uses the hex value, event without one falls back to the status color, unresolved event keeps `bg-base-300`
- [ ] 4.6 Render the category color on assignment cards in `timetable-cell.tsx` (inline style with luminance-based text color), satisfying the tests
