## Why

A Daylite project and its Planradar project are only connected in people's heads, so the planner cannot show or act on the Planradar side of an assignment.
Storing the Planradar project ID on the Daylite project gives every device the same link and is the base for creating (BL-037) and reactivating (BL-038) Planradar projects.

## What Changes

- Show in the assignment edit modal whether the assignment's Daylite project is linked to a Planradar project, with the linked project's name and status
- Let the planner pick an existing Planradar project from a name-filtered list and link it
- Store the Planradar project ID in the `planradar-link` custom field of the Daylite project
- Let the planner link another project when the stored link matches no Planradar project

## Capabilities

### New Capabilities
- `planradar-project-linking`: Show and set the link between a Daylite project and an existing Planradar project

### Modified Capabilities

None.

## Impact

- Backend: a new area above `daylite` and `planradar` that combines both, Daylite project read and write of the link field
- Frontend: Planradar section in the assignment edit modal
- Depends on BL-009 with its open project-ID follow-up, and on a way to connect Planradar in the settings, which no change covers yet
