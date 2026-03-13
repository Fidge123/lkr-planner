## Why

When planning a project that exists in Daylite but not in Planradar, users need a way to create the Planradar project and link it to the Daylite project. This enables unified planning across both systems.

## What Changes

- Allow creation of new Planradar project from unlinked Daylite project
- Present template selection from existing Planradar projects
- Persist created Planradar project ID to Daylite custom field
- Ensure idempotent operation (no duplicate creation on repeated runs)

## Capabilities

### New Capabilities
- `planradar-project-creation`: Create and link Planradar projects from Daylite

### Modified Capabilities
- `project-comparison`: Extended to include create option for unlinked projects

## Impact

- Code: New service function for project creation, UI flow for template selection
- Dependencies: Depends on BL-009 Planradar API client