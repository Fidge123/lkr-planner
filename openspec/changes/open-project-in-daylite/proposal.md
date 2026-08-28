## Why

A planner looking at an assignment card in the grid has the Daylite project reference right there, but reaching the project itself means switching to Daylite and searching for it by name.
The card already carries everything needed to jump straight to the record, and Daylite for Mac accepts a documented deep link that does exactly that.

## What Changes

- Assignment cards in the planning grid gain an action button on their right edge that opens the card's Daylite project in the Daylite macOS app.
- The button is icon-only, borderless and transparent, and sits in an action area on the card so a second card action can join it later without further layout work.
- The button is shown only on assignment cards whose Daylite project reference resolved.
  Bare events, absences, and assignments whose project could not be read carry no button.
- A new Rust command builds the `daylite://Command=ShowObject&Entity=Project&ID=<id>` URL from the card's project reference and hands it to the system, following ADR 0001: the frontend passes the reference and never constructs the URL.
- Whether Daylite is installed is not checked.
  An unregistered URL scheme is left to macOS, which reports it in its own dialog.
- Pressing the action button neither opens the edit modal nor starts a drag.
- The iCal `DESCRIPTION` stays `daylite:/v1/projects/<id>` as it is today.
  The deep link is built when the button is pressed and is never written to the calendar.

## Capabilities

### New Capabilities

- `daylite-deep-link`: opening a Daylite record in the Daylite macOS app from the planning grid, covering the URL contract, which cards offer the action, and how the action coexists with the card's click and drag behavior.

### Modified Capabilities

None.

## Impact

- Backend: a new `daylite_open_project` command in `src-tauri/src/integrations/daylite/`, registered in `src-tauri/src/lib.rs`.
- Frontend: `src/app/components/timetable-cell.tsx` (the card gains an action area), and a service wrapper alongside the existing Daylite services.
- Generated Tauri bindings in `src/generated/tauri.ts` are regenerated.
- No new Rust or npm dependency: `tauri-plugin-opener` is already a dependency and `lucide-react` already supplies the icon.
- No capability scope change in `src-tauri/capabilities/default.json`.
  The plugin enforces its URL scope inside its own IPC command, which this change does not use.
- macOS only in effect.
  On other platforms the button is still rendered and the URL still opened, and the platform decides what happens.
