## Decisions

### Link storage
The link is the Planradar project ID in the Daylite custom field `planradar-link`, so it travels with the project to every device.
Daylite documents `extra_fields` on a project as writable but gives neither its shape nor a way to define a field, so the first task proves reading and writing against the live API and whether the field has to be set up once in Daylite.
- A `planradar:<id>` keyword is writable and searchable today but shows up in Daylite's keyword list; it is the fallback if the proof fails.
- A mapping in the local store would not reach other devices.

### Stored ID form
Planradar answers with hashed project IDs while the recorded request paths use numeric ones.
The stored value is the form that the project read, copy and archive paths accept, which BL-009's follow-up 8.1 establishes before this change starts.

### Backend area
`daylite` and `planradar` sit on one layer and may not use each other, so linking lives in a new area above both, recorded in [ADR 0023](../../../docs/adr/0023-backend-areas-in-layers.md).
- Putting it into `calendar` would mix link handling into CalDAV code.

### Project list
The Planradar list endpoint has no text filter, so all pages are loaded once and filtered by name in the app.
- Reading Planradar on every keystroke would exhaust the client's 15 requests per minute.
