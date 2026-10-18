## Why

Reaching a week further away means clicking "Zurück" or "Weiter" once per week, or swiping once per week, which gets tedious for a planner checking an assignment a month back or preparing one six weeks ahead.
The calendar week label in the grid's top-left corner already tells the planner where they are, so it is the natural place to ask to go somewhere else.

## What Changes

- The calendar week cell in the grid's header corner becomes a control that opens a week picker anchored to it.
- The picker shows two months side by side, each as a grid of weeks with the calendar week number in front of every row.
- By default it shows the month of the displayed week and the month before it, which on opening the app is the current and the last month.
- Two buttons in the picker move the two-month window one month back or forward, so any other date can be reached.
- Choosing a day or a calendar week number jumps the planning view to that week and closes the picker.
- The displayed week and today are marked in the picker.
- The picker closes without changing the week on Escape or a click outside it.
- The existing "Zurück", "Heute" and "Weiter" buttons and the swipe gesture are unchanged.

## Capabilities

### New Capabilities

- `week-jump-navigation`: jumping the planning view directly to an arbitrary calendar week from a month-based week picker opened on the calendar week cell.

### Modified Capabilities

None.
The swipe gesture and the header navigation keep their behavior, and a jump loads its week the same way any other week change does.

## Impact

- Frontend: `src/app/components/week-table.tsx` turns the calendar week cell into the picker's trigger, a new `src/app/components/week-picker.tsx` renders the picker, and pure month and week calculations join `src/app/util.ts`.
- `src/app.tsx` gains a jump handler that sets the week offset directly, passed through `PlanningGrid` in `src/app/page.tsx`.
- The incoming week drawn during a swipe renders the calendar week cell without the picker.
- No backend change, no Tauri command and no new dependency: the picker is built from DaisyUI classes and Lucide icons already in use.
