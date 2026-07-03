Work red/green: write the failing test first, then the implementation that makes it pass.

## 1. Create-only ghost trigger

- [ ] 1.1 Write failing test: creating an assignment sets a ghost of the created project on the next visible day for that employee
- [ ] 1.2 Extend the `onSave` callback to carry the created project (reference plus name); set the row ghost state `{ date, project } | null` on create only
- [ ] 1.3 Write failing test: editing an assignment does not set a ghost
- [ ] 1.4 Ensure the edit and delete save paths call `onSave` without the created-project payload so no ghost is set

## 2. Target day and suppression

- [ ] 2.1 Write failing test: the ghost lands on the next visible day, and no ghost appears when saving on the last visible day
- [ ] 2.2 Derive the target day from the `weekDays` index after the created day; render the ghost only on the matching cell
- [ ] 2.3 Write failing test: the ghost is suppressed when the target day already holds any event for that employee
- [ ] 2.4 Evaluate suppression at render against live cell events (`cell.date === ghost.date && cell has no events`)

## 3. Ghost rendering

- [ ] 3.1 Write failing test: `TimetableCell` renders a `suggestion` prop with reduced opacity (~50%) and a dashed border, between the events and the `+` button
- [ ] 3.2 Add the optional `suggestion` prop to `TimetableCell` and render it accordingly

## 4. One-click add and chaining

- [ ] 4.1 Write failing test: clicking the ghost creates a persisted assignment for the target day and reloads the table
- [ ] 4.2 Wire the ghost click to reuse the create path (`createAssignment` plus reload)
- [ ] 4.3 Write failing test: after clicking, a new ghost of the same project appears on the following visible day (chaining)
- [ ] 4.4 Write failing test: clicking a ghost on the last visible day creates the assignment and shows no further ghost
- [ ] 4.5 Ensure the click sets the next ghost through the same create trigger so chaining honors suppression and the week boundary

## 5. Ghost clearing

- [ ] 5.1 Write failing test: deleting any assignment clears the ghost
- [ ] 5.2 Write failing test: navigating to another week clears the ghost
- [ ] 5.3 Write failing test: cancelling a modal without saving keeps the ghost
- [ ] 5.4 Clear the ghost on delete and on week change; leave it untouched on cancel
