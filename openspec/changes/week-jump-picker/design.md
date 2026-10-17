## Context

`App` in `src/app.tsx` owns the displayed week as `weekOffset`, a count of weeks relative to the week `getWeekDays(0, showWeekend)` resolves to.
That baseline is not always today's ISO week: with the weekend hidden, a Saturday or Sunday resolves offset 0 to the coming Monday.
`onNavigateWeek` only accepts `-1 | 1`, and the swipe gesture and the drag's edge hovering both depend on that signature.

The calendar week cell is a plain `<th>` in `WeekTable`, which is rendered twice while a swipe runs: once for the displayed week and once for the incoming week, inside a `pointer-events-none` layer.
The table itself also turns `pointer-events-none` during an appointment drag.

Component specs render with `renderToStaticMarkup`, so no click, key or effect can be exercised until `add-dom-component-testing` lands.

## Goals / Non-Goals

**Goals:**

- Keep all date arithmetic in pure, unit-tested functions so the untestable interactive part of the picker stays thin.
- Reuse the existing week-change path, so a jump triggers the same loading, prefetch and highlight handling as a step.

**Non-Goals:**

- Keyboard navigation between days inside the picker beyond the native tab order of its buttons.
- Animating the jump like a swipe.
- Choosing a date range or anything other than a whole week.

## Decisions

### Jump by setting the offset, not by a new date state

`App` gains `handleJumpToWeek(day: Date)`, which converts the chosen day to an offset with a pure `weekOffsetFor(day, showWeekend)` and calls `setWeekOffset`.
The function takes the Monday of the chosen day's ISO week and the Monday of `getWeekDays(0, showWeekend)`, and rounds their distance to whole weeks so a DST change inside the span cannot produce a fractional result.
Keeping `weekOffset` as the single source of truth leaves "Heute", the swipe and the drag untouched.
Widening `onNavigateWeek` to arbitrary offsets was rejected, because its `-1 | 1` type is what the swipe direction and the incoming week are derived from.

### Anchor the default months on the displayed week's Thursday

The right-hand month is the month of the displayed week's Thursday, the left-hand month is the one before it.
The Thursday is the day that decides an ISO week's year, and choosing it guarantees that every day of the displayed week is visible in one of the two months, whichever month the week starts in.
Anchoring on the Monday was rejected because a week starting on 28 September would open on August and September and hide the days in October.
The picker's own month state is reset to this anchor every time it opens.

### Pure month grid model

`monthWeeks(year, month)` in `src/app/util.ts` returns the month's rows, each a Monday-first array of seven dates plus its ISO week number from the existing `getIsoWeek`.
The rows start at the Monday on or before the 1st and end at the Sunday on or after the last day, so a month has four to six rows.
German month names come from `Intl.DateTimeFormat("de-DE", { month: "long" })`, matching how `TimetableHeader` formats day names.

### Native popover anchored to the cell

The cell content becomes a `<button>` with `popovertarget`, and the picker is a `popover="auto"` element styled as a DaisyUI `card`.
Its `beforetoggle` handler places it below the button from the button's bounding rect, because CSS anchor positioning needs a newer WebKit than every supported macOS ships.
`popover="auto"` gives light dismiss on Escape and outside click without a document listener, and renders in the top layer, so the sticky header's stacking context and the grid's scroll container cannot clip it.
A `details`/`summary` dropdown was rejected because it neither closes on outside click nor escapes the table's overflow on its own.
WKWebView on the supported macOS versions ships the Popover API.
Choosing a week calls `hidePopover()` before handing the day to `onJumpToWeek`.

### Picker only on the displayed week

`WeekTable` takes an optional `onJumpToWeek`.
When it is absent the cell renders as today, so the incoming week drawn during a swipe never mounts a second popover with the same id.
The popover id is derived with `useId` to stay unique regardless.

### Days and week numbers as buttons

Every day and every week number is a `<button>`, with an `aria-label` carrying the full German date or "KW n", so the static markup tests can assert on labels and the picker is reachable by tab.
Days of neighbouring months stay clickable and only get `text-base-content/40`, since jumping to them is the same action.
The displayed week's row uses `bg-primary/10`, and today gets an outlined primary emphasis consistent with the header's "Heute" button.

## Risks / Trade-offs

- [The popover cannot be exercised by `renderToStaticMarkup`] → The date logic is covered by unit tests in `util.spec.ts`, the rendered grid by static markup tests on `WeekPicker` with a fixed month, and the click flow is verified manually in the app until the DOM harness exists.
- [A fixed position does not follow the trigger when a grid wider than the window scrolls horizontally under the open picker] → The picker hides itself on a scroll of the grid's scroll container, which counts as closing without a jump.
- [Jumping during an appointment drag] → Not possible, because the table is `pointer-events-none` while a drag runs.
- [`weekOffsetFor` disagrees with `getWeekDays` on weekends] → Unit tests pin Saturday and Sunday system times with the weekend both hidden and shown.
