## Why

The planning view currently shows only the work week (Monday to Friday), with no way to plan or review assignments that fall on Saturday or Sunday.
Some teams occasionally work weekends and need to see those days, while most prefer an uncluttered Monday to Friday view.

## What Changes

- Add a display toggle "Wochenende anzeigen" in the settings dialog under "Anzeige".
- When the toggle is off (default), the planning view shows Monday to Friday (5 columns), matching today's behavior.
- When the toggle is on, the planning view shows Monday to Sunday (7 columns), adding Saturday and Sunday.
- Persist the toggle in the local store as part of the existing display settings so the choice survives restarts.

## Capabilities

### New Capabilities
- `weekend-visibility`: Persisted display setting controlling whether the planning view shows Saturday and Sunday, defaulting to off.

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: `getWeekDays` (week-day generation) becomes weekend-aware in both column count and current-week anchoring (with weekend on, today's Saturday or Sunday stays visible instead of jumping to the upcoming Monday), the planning page reads the setting, and the settings dialog gains a toggle.
- Behavior change: fixes an existing bug where `saveHideNonPlannableEmployees` overwrites the whole `displaySettings` object; both save helpers now merge so saving one display field no longer drops the other.
- Storage: New `showWeekend` field on the existing `DisplaySettings` in the local store (Rust + generated TypeScript types).
- Dependencies: Builds on the existing display-settings persistence used by `hideNonPlannableEmployees`.
