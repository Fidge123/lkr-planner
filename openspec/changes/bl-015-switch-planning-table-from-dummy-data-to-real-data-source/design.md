## Context

The weekly planning view uses hardcoded assignment fixtures. Users cannot persist their planning across app restarts. This change connects the planning table to real data: employee primary CalDAV calendars (hosted in ZEP), which serve as the source of truth for assignments.

During rollout, employees' primary calendars already contain manually-created legacy plan entries and personal blockers (e.g. car service appointments). These appear automatically as bare events with no Daylite project link, with no migration required.

## Goals / Non-Goals

**Goals:**
- Replace all dummy assignment data with data read from employee CalDAV calendars
- Display two tiers of events: lkr-planner assignments (project-linked) and bare events (legacy/blockers)
- Resolve Daylite project details for display (name, status/color) with API fallback
- Maintain existing loading/empty/error states in German
- Support week-to-week navigation with live calendar data

**Non-Goals:**
- Writing assignments to CalDAV (covered by BL-016 + BL-017)
- Assignment modal behavior (covered by BL-016, BL-031, BL-032, BL-033)
- Absence calendar display (follow-on scope)

## Decisions

### Source of Truth: CalDAV as Primary Storage
**Decision**: Employee primary CalDAV calendars are the source of truth for assignments
- No separate assignment store in LocalStore JSON
- lkr-planner reads events directly from ZEP CalDAV per visible week
- Legacy manually-created events appear automatically as bare events on day one of rollout
- Employee-created blockers appear automatically as bare events

### Event Encoding: DESCRIPTION Property
**Decision**: Encode Daylite project reference in the first line of the DESCRIPTION property
- Format: `daylite:/v1/projects/3001` as the first line of DESCRIPTION
- Subsequent lines are free-form notes
- Standard iCal property (RFC 5545), survives CalDAV round-trips on all major servers
- Visible and editable in all major calendar apps (Apple Calendar, Google Calendar, Outlook)
- X- custom properties are avoided (not reliably preserved by all CalDAV servers including ZEP)

### Two-Tier Event Display
**Decision**: Distinguish lkr-planner events from bare events visually
- **lkr-planner event**: DESCRIPTION first line matches `daylite:/<path>` → colored by project status, shows edit affordance
- **Bare event**: no structured project link → neutral/grey styling, read-only
- Covers legacy plans, employee-created blockers, and any unlinked calendar entries

### Daylite Project Resolution
**Decision**: Resolve project details via cache with API fallback, German placeholder on failure
- First: look up project reference in `dayliteCache` (LocalStore)
- Fallback: query Daylite API directly if not found in cache
- If API call fails: display German placeholder `"Beschreibung für [event SUMMARY] konnte nicht abgerufen werden"`
- Color: derive from project status if resolved; neutral if placeholder

### CalDAV Fetch Strategy
**Decision**: One CalDAV REPORT (calendar-query) per employee for the visible week
- Date range: Monday 00:00 to Sunday 23:59 of the visible week (derived from weekOffset)
- Parallel fetches across all employees
- Employees with no primary calendar configured: show empty row (no error, no fetch)

## Risks / Trade-offs

- **Risk**: ZEP CalDAV server unavailable
  - **Mitigation**: Show German error banner with retry button; grid shows partial data for employees whose fetch succeeded before failure

- **Risk**: Daylite project not in cache and API unreachable
  - **Mitigation**: German placeholder with event SUMMARY as fallback label; neutral color

- **Risk**: DESCRIPTION encoding scheme collides with user notes starting with `daylite:`
  - **Mitigation**: Acceptable edge case; users editing raw event data are expected to understand the format
