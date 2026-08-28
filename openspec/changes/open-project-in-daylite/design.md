## Context

See proposal.md - Why.

Four properties of the current code shape this design.

The card already holds the reference.
`CalendarCellEvent.projectRef` reaches the frontend as `/v1/projects/<id>`, written into the VEVENT `DESCRIPTION` by `build_ical_payload` and read back by `classify_event`.
`CellEvent.projectStatus` is set only when the project was resolved against Daylite, which is what `isUnresolvedAssignment` in `src/app/types.ts` tests.

The assignment card is itself a `<button>`.
`DraggableAssignmentCard` renders one button carrying both the click that opens the edit modal and the dnd-kit drag listeners.
A second control cannot be nested inside it.

`tauri-plugin-opener` is already a dependency but only reachable from Rust.
The npm package `@tauri-apps/plugin-opener` is not installed, and the plugin's injected click handler hard-codes the schemes it intercepts to `http:`, `https:`, `mailto:` and `tel:`.
A plain `<a href="daylite://...">` is therefore not intercepted at all, and the webview drops the navigation.

Daylite's published documentation is out of date on the deep link.
Marketcircle's interactive reports documentation gives `daylite4://ShowObject/<Entity>/<ObjectID>`.
That form no longer works.
The form a current Daylite accepts, confirmed by hand against the installed app, is `daylite://Command=ShowObject&Entity=Project&ID=<id>`.
No parameter for anything else is known, tab handling included.

## Goals / Non-Goals

**Goals:**

- The URL format lives in one place in Rust, so a future Daylite scheme change or a second entity type is a change to one function.
- The action is added without changing how the card is clicked or dragged, so no existing grid behavior has to be re-tested for regressions.
- The action area is a container from the start, so the second card action is a sibling rather than a re-layout.

**Non-Goals:**

- Detecting whether Daylite is installed.
- Deep links to anything other than a project.
- Reaching Daylite from the assignment modal, from a bare event, or from anywhere outside the grid card.
- Influencing whether Daylite reuses a tab.
  It is not exposed by the URL scheme, so it is Daylite's preference to make, not ours.

## Decisions

### The URL form comes from testing, not from the documentation

`daylite://Command=ShowObject&Entity=Project&ID=<id>` was arrived at by trying it against the installed Daylite.
The `Entity` and `Command` names carry over from the documented `daylite4://` form, and the id is the numeric segment of the `/v1/projects/<id>` reference.

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

### An overlaid action area, not a restructured card

The action area is a sibling of the card button inside the existing `<li>`, positioned over the card's right edge, and the card button gains right padding so its title stops before the area begins.

Nesting is not available: a `<button>` inside a `<button>` is invalid, and the card button is where both the modal click and the drag listeners sit.
Being a sibling rather than a descendant also means dnd-kit never sees the pointer events on the action, so no `stopPropagation` guard is needed to keep a press on the action from starting a drag.

Alternative considered and rejected: splitting the card into a flex row whose label is the button and whose actions are siblings in normal flow.
That moves the drag handle and the click target onto a smaller element, which changes drag behavior across the whole grid to add one button.

The card's `lifted` state hides the card button while a drag is in flight, and the action area has to be hidden by the same flag.
Otherwise it would be left visible in an `<li>` collapsed to zero height, floating over the row below.

### Gate on the resolved project, not on the reference alone

The action is rendered when `event.kind === "assignment"` and `isUnresolvedAssignment(event)` is false, which is the same predicate the grid already uses to decide whether a card is draggable.

The deep link would technically work for an unresolved assignment, since it needs only the id and not the Daylite API.
Showing it anyway was considered and rejected: an unresolved card already tells the planner that this reference could not be read, and offering a jump from a card in that state invites the reading that the app knows more about the project than it is showing.

### `ExternalLink` from Lucide

The codebase already uses `ExternalLink` for the one control that leaves the app, the token link in `daylite-panel.tsx`, so the icon keeps its established meaning of "this opens somewhere else".
`Link` reads as a chain and is the more literal icon for a link, but it is used in most icon sets for creating or holding a link rather than following one.
Swapping it is a one-line change if the second card action makes a different pairing look better.

### The deep link is never persisted

The VEVENT `DESCRIPTION` keeps `daylite:/v1/projects/<id>`.
It is load-bearing for `classify_event`, which reads the first description line to tell an assignment from a bare event, and it is what every write path reproduces.
The deep link is derived when the button is pressed.

## Risks / Trade-offs

The URL form rests on a manual test rather than on documentation, so a future Daylite release could change it with nothing to warn us.
The documented form has already gone stale once, which is how this one was found.
→ Keep the URL in one Rust function so a later correction is a one-line change, and keep the format asserted in a unit test so the expected string is written down somewhere a reader will find it.

The numeric id in the REST reference is taken to be the internal id the URL scheme expects.
The shapes match and the link resolves, but the test that established the format may have used an id read out of Daylite rather than out of the API.
→ Confirm once that an id taken from a `/v1/projects/<id>` reference opens that same project, before building on it.

`open_url` launches detached, so a URL that no application handles produces no error the app can catch.
→ Accepted by decision.
macOS shows its own dialog, and building an install check to replace it is explicitly out of scope.

The action overlays the card rather than sharing its flow, so a very narrow column could put the icon over wrapped title text.
→ The card button's right padding reserves the width, and a title that no longer fits wraps instead of running under the icon.
Worth a look at the narrowest column the grid produces.

## Migration Plan

None.
The change is additive: no stored data, no calendar payload, and no existing command contract changes, so it needs no migration and is reverted by removing the command and the button.

## Open Questions

Whether the two card actions, once the second one exists, still read clearly as icons alone or want a tooltip.
It changes neither the specs nor the task breakdown and is better answered with both buttons on screen.
