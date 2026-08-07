## Context

See proposal.md - Why.

Four properties of the current write path shape this design.

An assignment event is rebuilt from scratch on every modal save and every drag.
`build_ical_payload` emits a fixed VEVENT from UID, date, project name, project reference, slot times, and order index; whatever else the resource held is gone after the PUT.
Only slot re-allocation preserves foreign properties, because `patch_event_slot` copies through every line it does not rewrite.

`DESCRIPTION` is load-bearing.
`classify_event` reads the Daylite project reference from the first description line, and an event whose first line does not start with `daylite:` is a bare event, not an assignment.

`SUMMARY` is written but not read back for assignments.
`resolve_event` overwrites the title with the resolved Daylite project name and only falls back to `SUMMARY` when resolution fails, which is what makes a project rename in Daylite show up on the card.

`patch_event_slot` is line-based and refuses to patch an event whose rewritten property (`DTSTART`, `DTEND`, `X-LKR-ORDER`) is followed by a folded continuation line.
An event it refuses is silently excluded from slot re-allocation.

## Goals / Non-Goals

**Goals:**

- One place in the VEVENT for each new field, chosen so a planner editing the event in ZEP or another CalDAV client sees something sensible.
- No write path can silently drop a field: the fields travel together through `AssignmentWrite`, so adding one to the struct forces every caller to supply it.
- The written VEVENT stays patchable by `patch_event_slot`, so assignments with attachments still take part in slot re-allocation.

**Non-Goals:**

- Editing start and end time.
  The slot allocator owns the day's time windows and hand-edited times would be overwritten by the next write on that day.
- Server-managed attachments (RFC 8607).
  ZEP's CalDAV server is not known to implement it and the feature would be unavailable wherever it does not.
- Attachment versioning, deduplication, or an attachment browser outside the modal.
- Editing bare events or absences.
  They stay read-only, as they are today.

## Decisions

### The custom title lives in `SUMMARY`, `X-LKR-TITLE` keeps the title it replaced

`SUMMARY` carries whatever title the planner wants shown, because that is the property every calendar client reads.
`X-LKR-TITLE` carries the title the custom one replaced, which is the Daylite project name at the moment the planner typed over it.
Its presence is the marker that `SUMMARY` was set by hand: `resolve_event` uses `SUMMARY` when the property is there and the resolved project name when it is not, so an assignment nobody has renamed keeps following the project through a rename in Daylite exactly as today.
Writing an assignment without a custom title emits no `X-LKR-TITLE` and puts the project name in `SUMMARY`, as today.

Holding the replaced title rather than a copy of the custom one is what makes a later reset affordance cheap: the modal can offer "Titel zurücksetzen" and show the planner what it would go back to without resolving anything.
That affordance is not in this change - the planner resets by emptying the field, which drops `X-LKR-TITLE` and restores the project name - but the storage is chosen so adding it later needs no format change.

One consequence to keep in mind: the stored value ages.
A project renamed in Daylite after the planner set a custom title leaves `X-LKR-TITLE` holding the old name.
So it is a record of what was replaced, not a cache of the project name, and nothing reads it as one.
Reset therefore clears the property and falls back to live resolution rather than writing the stored value back into `SUMMARY`, and the stored value is only ever displayed as the fallback when resolution fails.

Alternatives considered.
Mirroring the custom title into both properties makes the marker redundant with `SUMMARY` and throws away the replaced title for no gain.
Storing the custom title in the description below the `daylite:` line mixes it with the planner's note and forces a parsing convention on text a human also edits.
A bare boolean marker property would work for display, but loses the reset affordance for the same number of bytes on the wire.

### The note is the description below the `daylite:` line

`DESCRIPTION` becomes `daylite:<ref>` followed by a blank line and the note.
`classify_event` already reads only the first line, so classification is unaffected, and a note written by another calendar client below the reference line is picked up as the assignment's note.
The note is what remains after dropping the first line and any single blank line that follows it, so a round-trip through the planner does not accumulate blank lines.

Alternatives considered.
An `X-LKR-NOTE` property keeps the description clean but hides the note from every other calendar client, which is the main reason to store it on the event at all.
Moving the project reference to an X-property and giving `DESCRIPTION` entirely to the note would be cleaner, but every assignment already on a calendar carries the reference in the description, so it would need a migration and would break older events.

### Attachments are inline base64 `ATTACH` properties, capped at 5 MB per event

Each attachment is one `ATTACH;ENCODING=BASE64;VALUE=BINARY;FMTTYPE=<mime>;FILENAME=<name>` property.
`FILENAME` is a non-standard parameter, but it is what calendar clients converged on for inline attachments and it is the only way to keep the original name.
The cap is on the total base64 payload of one event, checked in the frontend before a file is added and again in the backend before the PUT.
5 MB keeps a re-PUT of the whole event well inside what CalDAV servers accept, and every write path re-PUTs the whole event.

This was the recommended option in the question put to the user; the question went unanswered, so the design proceeds on it.
The alternatives, both still open if the cap or the interoperability trade-off turns out wrong: storing files in the local store keyed by event UID, which removes the size problem but makes attachments invisible in ZEP and on a second machine, and storing a URI instead of the bytes, which is cheapest but only works while the target path stays reachable.

Files are read through a browser file input in the webview and passed to the backend as bytes, and an opened attachment is written to a temp file and handed to the already-installed opener plugin.
That avoids adding `@tauri-apps/plugin-dialog` and its capability entry for a native file picker; the trade-off is the webview's file dialog instead of a fully native one.

### Property order in the payload keeps events patchable

Long lines are folded per RFC 5545, which is a change on its own: `build_ical_payload` writes unfolded lines today, and a base64 attachment cannot stay unfolded.
Because `patch_event_slot` refuses an event whose folded line follows a rewritten property, the payload is emitted as `UID`, `DTSTAMP`, `DTSTART`, `DTEND`, `X-LKR-ORDER`, `SUMMARY`, `X-LKR-TITLE`, `DESCRIPTION`, `ATTACH...`.
`X-LKR-ORDER` moves ahead of the foldable properties so no fold ever follows `DTSTART`, `DTEND`, or `X-LKR-ORDER`.

### The new fields travel in `AssignmentWrite`

`AssignmentWrite` grows `title_override: Option<String>`, `note: Option<String>`, and `attachments: Vec<Attachment>`.
Every write path already builds one, so the compiler flags each caller that has to be taught to carry the fields through: the modal save, `move_assignment`, and the drag hook's reschedule.

The drag paths have no dialog to read the fields from, so they must carry the values the card already holds.
`CalendarCellEvent` therefore gains `titleOverride`, `note`, and an attachment list, and the drag hook passes them back into the write.
Attachments are the awkward case: shipping their bytes to the frontend just so a drag can ship them back is wasteful, so the frontend carries only each attachment's metadata and the backend re-reads the bytes from the source event during `move_assignment`.

### Reorder needs no change

`reorder_assignment_core` never rebuilds the event; it re-slots the day through `patch_event_slot`, which copies unknown properties through.
Attachments and the new properties survive it as long as the fold rule above holds.

## Risks / Trade-offs

[A calendar server rejects an event carrying a multi-megabyte attachment] → The cap keeps events small, and the specs require the server's refusal to surface as a German error with the modal left open, so the planner can remove an attachment and retry.

[`patch_event_slot` refuses an event whose attachment folds directly after a rewritten property, silently dropping it from re-allocation] → Property order is pinned by the design and covered by a test that re-slots an event with an attachment; an event written by another client can still be refused, which is the existing behavior for such events.

[The `icalendar` crate may not unescape `\n` in `DESCRIPTION` the way `classify_event`'s `lines()` assumes] → Verified by a test before the note is built on top of it; if it does not, unescaping moves into the parse step.

[A large base64 attachment crosses the Tauri IPC boundary on every open] → Bounded by the same 5 MB cap; opening writes a temp file rather than streaming the bytes into the webview.

[`FILENAME` on `ATTACH` is not in RFC 5545, so a strict server may strip it] → An attachment whose name is missing falls back to a generated name from its content type; the bytes are what matter.

[A planner expects an attachment to be visible in ZEP and it is not, because the server does not surface inline attachments] → Not mitigable in the planner; if it happens, it is the trigger to revisit the storage decision above.

## Migration Plan

No data migration.
Events written before this change carry no `X-LKR-TITLE`, no note, and no `ATTACH`, and every new field is optional, so they load and save exactly as they do today.
An older build of the planner reading an event written by the new one shows the override as the card title only when the project fails to resolve, and rewrites the event without the note and attachments - so a rollback loses details on the assignments touched after it, which is the reason the fields are stored on the event rather than only in the local store.
