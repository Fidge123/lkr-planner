# ADR 0008: Daylite Employee Contacts and URLs-Only iCal Mapping

- Status: Accepted
- Date: 2026-02-20

## Context

BL-008 requires employee data to come from Daylite contacts, filtered to actual field employees, and requires two iCal references (assignment + absence) to be readable and writable in Daylite.

The app already has:
- Daylite contact read/search commands in Rust
- A local persisted cache store for restart-safe rendering
- Frontend planning table still using dummy employee data

A previous fallback path for iCal extraction from `extra_fields` exists in the frontend domain mapper.

## Decision

- Use Daylite contacts as employee source in the planning view.
- Filter employees strictly by contact `category == "Monteur"` (case-insensitive).
- Keep iCal mapping on Daylite contact `urls` only:
  - primary assignment iCal URL
  - secondary absence iCal URL
- Remove iCal extraction/mapping via `extra_fields` from the frontend domain mapper.
- Add Daylite write support for iCal URLs via a dedicated Tauri command that updates `/contacts/{id}` using `urls` payloads.
- Extend persisted contact cache entries with contact fields needed for immediate restart rendering (name/category/urls).

## Consequences

- Planning employee rows are now backed by Daylite contact data instead of static dummy employees.
- Employees can be displayed immediately after restart from persisted cache while fresh data loads.
- iCal read/write behavior is explicit and consistent by using `urls` only.
- Existing Daylite setups that only stored iCal references in `extra_fields` will not be read by default anymore and should be migrated to `urls`.
