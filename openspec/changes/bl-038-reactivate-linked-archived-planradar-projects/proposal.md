## Why

A linked Planradar project that was archived cannot take new tickets, so work planned for it cannot be documented there until someone reopens it in Planradar.
The planner already shows that the linked project is archived and is where the need comes up.

## What Changes

- Offer `Reaktivieren` in the assignment edit modal when the linked Planradar project is archived
- Reactivate the project in Planradar on that action and show it as active once Planradar confirms it
- Show a German error message when reactivation fails, a missing permission included
- Record the live reactivate exchange as a cassette

## Capabilities

### New Capabilities
- `planradar-project-reactivation`: Reactivate an archived linked Planradar project from the planner

### Modified Capabilities

None.

## Impact

- Backend: the linking area from BL-010, Planradar cassettes
- Frontend: Planradar section of the assignment edit modal
- Depends on BL-009 and BL-010
