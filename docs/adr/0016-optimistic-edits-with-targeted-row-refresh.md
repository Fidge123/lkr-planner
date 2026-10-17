# ADR 0016: Optimistic Edits with Targeted Row Refresh

- Status: Accepted
- Date: 2026-10-17

## Context

Every write from the assignment modal, the next-day quick add, or a drag reloaded the whole displayed week with the loading spinner shown.
A dropped card snapped back to its old cell until that reload finished, which took as long as a full week load.
Writes re-slot the affected days on the server, so the times of the edited card and its neighbours are only known once the backend has written them.

### Evaluated Options
- Patch the local week optimistically and refresh only the affected employees' rows afterwards
  - Pros: A drop or a delete is visible at once; the refresh costs two CalDAV requests per affected employee instead of two per employee in the week.
  - Cons: The grid briefly shows a state the server has not confirmed, so a failed or overlapping write needs a rule for which state wins.
- Return the re-slotted events of the affected days from every write command
  - Pros: The grid could show the server's state without a second request.
  - Cons: Every write command and its slot allocation would have to report every event it touched, a much larger backend change.
- Keep reloading the whole week, without the spinner
  - Pros: Smallest change.
  - Cons: The card still snaps back for the length of a full week load.

## Decision

- A drop moves the card into its target cell before the write starts, renumbers that cell, and clears the card's times until the backend has re-slotted the day.
- A delete removes the card once the backend confirms it.
- After any write, successful or not, `load_week_events` is called with the `refresh` purpose for the affected employees of the edited week, and their rows are replaced without the loading spinner.
- A refresh result is discarded if another optimistic edit happened after the refresh was requested, so an older refresh cannot undo a newer edit; the newer edit's own refresh applies.
- A failed refresh falls back to a full reload of the week if it is still shown, or drops the week from the cache otherwise.
- A drop into a week other than the source week discards the cached source week, so it is fetched again when shown.
- A new assignment gets no placeholder card; it appears with the refresh that follows its write.

## Consequences

- Drops and deletes are reflected immediately, and the follow-up refresh replaces only the touched rows.
- A rejected write shows its card in the target cell until the refresh puts it back, alongside the German error message.
- A moved card shows no times for the short time between the drop and the refresh.
