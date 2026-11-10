## Why

A Daylite project that has assignments but no Planradar project has to be created in Planradar by hand and then linked, which repeats the project name and the choice of template every time.
Creating it from the planner, blank or as a copy of an existing project, leaves the new project linked in one step.

## What Changes

- Offer `In Planradar anlegen` in the assignment edit modal for a Daylite project without a usable Planradar link
- Create a blank Planradar project from a form whose name defaults to the Daylite project name, and link it
- Create a project as a copy of an existing Planradar project with a choice of aspects, link it, then let the planner adjust the copy's details
- Refuse a name that an existing Planradar project already carries and offer to link that project instead

## Capabilities

### New Capabilities
- `planradar-project-creation`: Create a Planradar project for a Daylite project, blank or as a copy, and link it

### Modified Capabilities

None.

## Impact

- Backend: Planradar client (project details read and update), the linking area from BL-010
- Frontend: Planradar section of the assignment edit modal
- Depends on BL-009 and BL-010, and ships as two pull requests: blank creation, then copy
