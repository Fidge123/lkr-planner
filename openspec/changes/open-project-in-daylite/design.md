## Context

See proposal.md - Why.

Four properties of the current code shape this design.

The card already holds the reference.
`CalendarCellEvent.projectRef` reaches the frontend as `/v1/projects/<id>`, written into the VEVENT `DESCRIPTION` by `build_ical_payload` and read back by `classify_event`.
`CellEvent.projectStatus` is set only when the project was resolved against Daylite, which is what `isUnresolvedAssignment` in `src/app/types.ts` tests.

The assignment card is one button carrying two jobs.
`DraggableAssignmentCard` renders a single `<button>` holding both the click that opens the edit modal and the dnd-kit drag listeners.
Nothing can be placed inside it that is itself pressable, which is what this change unpicks.

Keyboard dragging is not wired up.
`page.tsx` registers only `PointerSensor`, so the `role="button"` and `tabIndex` that dnd-kit's `attributes` put on the card today buy no keyboard drag.
Dropping them costs nothing that works.

`tauri-plugin-opener` is already a dependency but only reachable from Rust.
The npm package `@tauri-apps/plugin-opener` is not installed, and the plugin's injected click handler hard-codes the schemes it intercepts to `http:`, `https:`, `mailto:` and `tel:`.
A plain `<a href="daylite://...">` is therefore not intercepted at all, and the webview drops the navigation.

Daylite's published documentation is out of date on the deep link.
Marketcircle's interactive reports documentation gives `daylite4://ShowObject/<Entity>/<ObjectID>`.
That form no longer works.
The form a current Daylite accepts, confirmed by hand against the installed app, is `daylite://Command=ShowObject&Entity=Project&ID=<id>`.
The id in it is the numeric segment of the `/v1/projects/<id>` API reference, so the card carries the identifier the link wants without a lookup.
No parameter for anything else is known, tab handling included.

## Goals / Non-Goals

**Goals:**

- The URL format lives in one place in Rust, so a future Daylite scheme change or a second entity type is a change to one function.
- The card's actions are siblings in the card's normal flow, so a third action is an added element rather than a re-layout.
- Dragging keeps feeling the same to a planner, and pressing an action can never be mistaken for the start of one.

**Non-Goals:**

- Detecting whether Daylite is installed.
- Deep links to anything other than a project.
- Making the card keyboard-draggable.
  It is not today, and this change neither adds nor removes that.
- Reaching Daylite from the assignment modal, from a bare event, or from anywhere outside the grid card.
- Influencing whether Daylite reuses a tab.
  It is not exposed by the URL scheme, so it is Daylite's preference to make, not ours.

## Decisions

### The URL form comes from testing, not from the documentation

`daylite://Command=ShowObject&Entity=Project&ID=<id>` was arrived at by trying it against the installed Daylite, with an id taken from a `/v1/projects/<id>` reference.

Two things about this string invite a well-meant correction that would break it.
It is not a query string: the parameters sit in the authority position, with no `?` after `daylite://`.
And the scheme is `daylite:`, not the `daylite4:` that Marketcircle's own documentation still shows.
Both are load-bearing, which is why the command builds the URL by formatting a string rather than through a URL type that would normalize it.

### A Rust command, not the opener plugin's own IPC command

`daylite_open_project(project_ref: String)` takes the reference, extracts the numeric id, builds the URL, and calls `app.opener().open_url(url, None::<&str>)`.
The frontend calls it through the generated bindings and never sees a URL.

This follows ADR 0001 and ADR 0009, and it makes the reference-to-URL translation a plain Rust unit test with no Tauri runtime and no HTTP cassette.

It also sidesteps the plugin's scope system, which is worth stating explicitly because the alternative looks cheaper than it is.
The URL scope is enforced inside the plugin's own `open_url` IPC command, not inside `OpenerExt::open_url`.
Calling the extension trait from our command is not scope-checked, so `src-tauri/capabilities/default.json` needs no entry.
Going through the JS package instead would have required adding the npm dependency and widening the capability with an `opener:allow-open-url` scope entry for `daylite:`, because `opener:default` expands to `allow-default-urls`, which covers only `mailto:`, `tel:`, `https://` and `http://`.

Alternative considered and rejected: returning the URL from Rust and opening it in the frontend.
It splits one operation across the boundary for no gain and still needs the scope entry.

### The card stops being a control, and its actions sit in its normal flow

The card becomes a container laying out three things in a row: the times and title as before, then the edit action, then the Daylite action.
Neither action is nested in anything pressable, so both are ordinary buttons and no propagation guard is needed to keep one from triggering the other.

Making the card a container rather than a button is what buys this.
While it was a button, the only way to add a second control was to overlay it as a sibling positioned over the card's right edge, with reserved padding on the card so the title did not run underneath, because a `<button>` cannot contain a `<button>`.
That arrangement worked for one action and got worse with each one added.
In normal flow the row measures itself, the title's `flex-1 min-w-0` shrinks around whatever the actions need, and the narrow-column overlap it would otherwise have risked cannot happen.

The cost is that the card no longer announces itself as pressable, and a planner used to clicking anywhere on a card has to find the pencil.
That is the change being asked for, and it is what makes the two actions distinguishable at all.

### `setNodeRef` on the card, `setActivatorNodeRef` on the body

dnd-kit separates the element it measures from the element that starts a drag.
`setNodeRef` goes on the card container, so the rect the drop logic and the overlay use still covers the whole card as it does today.
`setActivatorNodeRef` and the drag `listeners` go on the body, the times-and-title region, so a press on either action is not a press on the drag activator.

This is what keeps "pressing an action never starts a drag" true without a `stopPropagation` handler on each action, and it keeps the draggable area the part of the card a planner would grab anyway.

dnd-kit's `attributes` are not spread as they are today.
They carry `role="button"` and `tabIndex`, which on a card that is no longer a control would announce a control that does nothing.
`aria-roledescription` is kept so the card still describes itself as draggable.
Nothing is lost, because no `KeyboardSensor` is registered.

Alternative considered and rejected: leaving the listeners on the container and calling `stopPropagation` on each action's pointer-down.
It puts a guard on every action forever, and forgetting it on the third one is a bug that only shows up as an accidental drag.

### The edit action is shown on every assignment, the Daylite action is not

The edit action does not depend on Daylite being reachable, and withholding it would make an assignment whose project could not be read impossible to correct or delete.
So it is shown whenever the card is an assignment.

The Daylite action is shown only when `isUnresolvedAssignment(event)` is false.
The deep link would technically work for an unresolved assignment, since it needs only the id and not the Daylite API.
Showing it anyway was considered and rejected: an unresolved card already tells the planner that this reference could not be read, and offering a jump from a card in that state invites the reading that the app knows more about the project than it is showing.

### Icons and their order

Edit is Lucide `Pencil`, the Daylite jump is Lucide `ExternalLink`.
`ExternalLink` already carries the meaning "this opens somewhere else" in this codebase, on the token link in `daylite-panel.tsx`.
`Link` reads as a chain and is the more literal icon for a link, but in most icon sets it means making or holding a link rather than following one.

Edit comes first because it is the action a planner reaches for most, and it is the one every assignment card has.
Fixing the order at all is the point: a Daylite action that moves depending on whether it is present would put the pencil in a different place on neighbouring cards.

## Risks / Trade-offs

The card loses its click target, which is a habit change for every planner using the app today.
→ Nothing in the design softens it, by choice.
Worth watching after the first release rather than pre-empting with a transitional affordance.

The two icons sit on a card that can be narrow, and they are small pointer targets compared to the whole card.
→ They share the card's full height in the flex row, so the target is taller than the icon.
Check it against the narrowest column the grid produces.

The URL form rests on a manual test rather than on documentation, so a future Daylite release could change it with nothing to warn us.
The documented form has already gone stale once, which is how this one was found.
→ Keep the URL in one Rust function so a later correction is a one-line change, and keep the format asserted in a unit test so the expected string is written down somewhere a reader will find it.

`open_url` launches detached, so a URL that no application handles produces no error the app can catch.
→ Accepted by decision.
macOS shows its own dialog, and building an install check to replace it is explicitly out of scope.

## Migration Plan

None.
No stored data, no calendar payload, and no existing command contract changes.
The Daylite half is reverted by removing the command and the action; the card restructuring is reverted by putting the click back on the card.

## Open Questions

Whether the two actions read clearly as icons alone or want a tooltip.
It changes neither the specs nor the task breakdown and is better answered with both on screen.
