## Why

A planner looking at an assignment card in the grid has the Daylite project reference right there, but reaching the project itself means switching to Daylite and searching for it by name.
The card already carries everything needed to jump straight to the record, and Daylite for Mac accepts a deep link that does exactly that.

Adding that jump to a card that is itself one big button leaves nowhere to put it.
So the card gives up being a control, and both the jump and the edit it already had become named actions on its right edge.

## What Changes

- The assignment card stops being a button.
  Clicking its body no longer opens the edit modal and no longer does anything at all.
- Assignment cards gain an action area on their right edge holding icon-only, borderless, transparent controls.
- Editing moves into that area as the first action.
  It is shown on every assignment card, including one whose Daylite project could not be resolved, so an assignment stays editable and deletable whether or not Daylite can be reached.
- A second action in that area opens the card's Daylite project in the Daylite macOS app.
  It is shown only where the Daylite project reference resolved.
- Dragging a card is unchanged for the planner, but is activated from the card's body rather than from the whole card, so pressing an action never starts a drag.
- A new Rust command builds the `daylite://Command=ShowObject&Entity=Project&ID=<id>` URL from the card's project reference and hands it to the system, following ADR 0001: the frontend passes the reference and never constructs the URL.
- Whether Daylite is installed is not checked.
  An unregistered URL scheme is left to macOS, which reports it in its own dialog.
- The iCal `DESCRIPTION` stays `daylite:/v1/projects/<id>` as it is today.
  The deep link is built when the action is triggered and is never written to the calendar.

## Capabilities

### New Capabilities

- `daylite-deep-link`: opening a Daylite record in the Daylite macOS app from the planning grid, covering the URL contract, which cards offer the action, how it looks and where it sits, and its isolation from the card's other behavior.

### Modified Capabilities

- `assignment-persistence`: an assignment card is no longer a control, its actions sit in an action area on its right edge, and the edit affordance is one of them and is present even on an unresolved assignment.
- `assignment-modal-crud`: edit mode is opened from an assignment card's edit action rather than by clicking the card.
- `appointment-drag-drop`: a drag is activated from the card's body, and a press on a card action never starts one.

## Impact

- Backend: a new `daylite_open_project` command in `src-tauri/src/integrations/daylite/`, registered in `src-tauri/src/lib.rs`.
- Frontend: `src/app/components/timetable-cell.tsx` carries the whole card restructuring, and a service wrapper joins the existing Daylite services.
  `src/app/components/timetable-row.tsx` keeps passing the edit callback, now reached from the action rather than the card.
- Generated Tauri bindings in `src/generated/tauri.ts` are regenerated.
- Existing tests that open the modal by clicking an assignment card need updating, in `timetable-cell.spec.tsx` and in the grid-level specs that exercise the edit flow.
- No new Rust or npm dependency: `tauri-plugin-opener` is already a dependency and `lucide-react` already supplies both icons.
- No capability scope change in `src-tauri/capabilities/default.json`.
  The plugin enforces its URL scope inside its own IPC command, which this change does not use.
- macOS only in effect for the Daylite action.
  On other platforms it is still rendered and the URL still opened, and the platform decides what happens.
