## 1. Slot Allocation Function

- [ ] 1.1 Implement pure function `allocateSlots(assignments: Assignment[]): Slot[]`
- [ ] 1.2 Define types: `Assignment` (id, employeeId, date), `Slot` (assignmentId, startTime, endTime)
- [ ] 1.3 Sort assignments by ID for canonical ordering
- [ ] 1.4 Calculate equal slot duration: 480 minutes / assignment count

## 2. Edge Cases

- [ ] 2.1 Handle empty assignment list (return empty slots)
- [ ] 2.2 Handle single assignment (full window)
- [ ] 2.3 Handle many assignments (verify math handles integer division)

## 3. Testing

- [ ] 3.1 Write unit tests for 1 assignment (full window)
- [ ] 3.2 Write unit tests for 2 assignments (half window each)
- [ ] 3.3 Write unit tests for 3 assignments (third window each)
- [ ] 3.4 Write unit tests for reordered input producing identical output
- [ ] 3.5 Write unit tests for boundary times (08:00 start, 16:00 end)
- [ ] 3.6 Write unit tests for empty input