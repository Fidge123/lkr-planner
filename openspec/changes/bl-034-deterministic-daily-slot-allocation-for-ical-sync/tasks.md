## 1. Slot Allocation Function (Rust, pure)

- [ ] 1.1 Implement `allocate_slots(uids: &[String]) -> Vec<(String, NaiveTime, NaiveTime)>` (or equivalent) returning a slot per UID
- [ ] 1.2 Sort UIDs for canonical ordering independent of input order
- [ ] 1.3 Split the 480-minute 08:00-16:00 window evenly; use [start, end) half-open intervals
- [ ] 1.4 Handle empty input (return empty) and single assignment (full window)

## 2. Re-allocation on Write (BL-017 write paths)

- [ ] 2.1 After create: gather the employee's lkr-planner assignments for that day, allocate, and PUT updated times for events whose slot changed
- [ ] 2.2 After delete: re-allocate the remaining same-day assignments and PUT updated times
- [ ] 2.3 After update: re-allocate the target day; if the day changed, also re-allocate the source day
- [ ] 2.4 Exclude bare/absence/holiday events from re-allocation (only `daylite:/<path>` events)
- [ ] 2.5 Replace the fixed 08:00-16:00 window in `build_ical_payload` usage with allocated times

## 3. Testing

- [ ] 3.1 Unit tests for 1, 2, 3 assignments (full / half / third windows)
- [ ] 3.2 Unit test for reordered input producing identical output
- [ ] 3.3 Unit tests for boundary times (08:00 start, 16:00 end) and empty input
- [ ] 3.4 Write-path tests: create/delete/update redistribute the affected day(s)
- [ ] 3.5 Test that bare/absence/holiday events are never re-slotted
