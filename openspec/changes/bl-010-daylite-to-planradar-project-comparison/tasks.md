## 1. Link storage

- [ ] 1.1 Record a Daylite cassette that reads and writes the `planradar-link` field on a project, and switch design.md to the keyword fallback if that fails
- [ ] 1.2 Read the link from a Daylite project and write it, with unit tests

## 2. Linking commands

- [ ] 2.1 Give the linking area a layer above `daylite` and `planradar` in `architecture.rs` and ADR 0023
- [ ] 2.2 Add a command that returns a Daylite project's link state (unlinked, linked with name and status, not found), with unit tests
- [ ] 2.3 Add a command that lists all Planradar projects across pages, with unit tests
- [ ] 2.4 Add a command that links a Daylite project to a Planradar project and keeps a link another device stored, with unit tests

## 3. Edit modal

- [ ] 3.1 Add the service functions and query hooks, and render the Planradar section in its linked, unlinked, not-found and unavailable states, with component specs
- [ ] 3.2 Add the name-filtered project picker and the link action, with component specs
