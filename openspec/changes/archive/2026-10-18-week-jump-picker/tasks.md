# Tasks

## 1. Date calculations

- [x] 1.1 Add `monthWeeks(year, month)` to `src/app/util.ts`, returning Monday-first week rows with their ISO week number, red/green against `util.spec.ts` cases for October 2026 (first row starts 28 September, numbered 40), a month starting on a Monday, and a December row numbered 53 or 1
- [x] 1.2 Add `pickerAnchorMonth(weekDays)` returning the year and month of the displayed week's Thursday, red/green against cases for a week inside one month and the week of 28 September 2026
- [x] 1.3 Add `weekOffsetFor(day, showWeekend)` returning the offset that makes `getWeekDays` show the chosen day's ISO week, red/green against cases on a weekday, on a Saturday and a Sunday system time with the weekend hidden and shown, across a DST change and across the year boundary

## 2. Week picker component

- [x] 2.1 Create `src/app/components/week-picker.tsx` rendering two months with German month headings, "Mo" to "So" column labels, week number buttons labelled "KW n", day buttons labelled with the full German date, subdued neighbouring-month days, the displayed week's row highlighted and today marked, verified by `renderToStaticMarkup` tests in `week-picker.spec.tsx` with a fixed system time
- [x] 2.2 Add the previous and next month controls with German `aria-label`s and Lucide chevrons, moving both months by one across year boundaries, verified by a static markup test for a December and January start state and by month arithmetic tests in `util.spec.ts`
- [x] 2.3 Make the picker a `popover="auto"` element with a `useId` id that resets to the anchor months on `beforetoggle` open, positions itself below its trigger, hides itself on a scroll of the grid container and calls `hidePopover()` before reporting the chosen day, verified by a static markup test for the `popover` attribute and manually in the app

## 3. Wiring the jump

- [x] 3.1 Give `WeekTable` an optional `onJumpToWeek` and render the calendar week cell as a `popovertarget` button labelled "Woche auswählen" only when it is set, verified by `week-table.spec.tsx` tests for the cell with and without the prop
- [x] 3.2 Add `handleJumpToWeek` in `src/app.tsx` using `weekOffsetFor`, pass it through `PlanningGrid` in `src/app/page.tsx` to the displayed `WeekTable` only, and verify in `page.spec.tsx` that the grid renders the picker trigger when `onJumpToWeek` is passed (the incoming week's missing trigger is covered by 3.1)
- [x] 3.3 Verify in the running app that clicking "KW n" opens the picker on the displayed and previous month, month navigation works, choosing a day or week number jumps and closes, Escape and an outside click close without a jump, and the swipe and header buttons still work

## 4. Integration checks

- [x] 4.1 Run `bun test`, `bun run lint` and `bunx tsc` and verify all pass
- [x] 4.2 Run `bunx openspec validate week-jump-picker --strict` and verify the change is valid
