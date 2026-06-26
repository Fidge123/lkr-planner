## Why

The assignment modal needs live filtering as the user types. This allows quick project finding without navigating away from the modal.

## What Changes

- Build the combobox shell that replaces today's `<select>`: a filter input plus a result list
- The result list starts empty; filtered results appear once the user types at least 3 characters
- Show first 5 matching projects from `new_status` and `in_progress` states, sorted by project name
- Support keyboard selection (arrow keys + enter) over whichever list is displayed
- Clear filter returns the list to its empty default state (default-suggestion content is owned by BL-031)
- Add optional sort support to `search_projects_core` (default numeric ID, opt-in name sort)
- Add a new trailing-edge debounce hook for search-as-you-type

## Capabilities

### New Capabilities
- `assignment-modal-filter`: Live text filter for assignment modal

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New React combobox component (filter input + result list + keyboard nav), new trailing debounce hook, sort option added to `search_projects_core` / `DayliteSearchInput`; removes the `<select>` and the bulk `loadProjectsForAssignmentPicker` pre-load
- Dependencies: Uses Daylite query capabilities from BL-022
- Dependents: BL-031 plugs its default suggestions into this combobox's empty state; this change has no dependency on BL-031
