## 1. Type Definitions

- [ ] 1.1 Define `CalendarCellItem` interface with common fields
- [ ] 1.2 Define `CalendarCellItemSourceData` union type for source-specific data
- [ ] 1.3 Define item type enum: absence, holiday, assignment, appointment

## 2. Composition Function

- [ ] 2.1 Implement `composeCalendarCellItems()` function
- [ ] 2.2 Fetch and map absence calendar items
- [ ] 2.3 Fetch and map holiday items with German names
- [ ] 2.4 Fetch and map project assignments
- [ ] 2.5 Fetch and map preexisting appointments

## 3. Ordering and Flags

- [ ] 3.1 Implement sort by type then start time
- [ ] 3.2 Set isReadOnly flag for each item type
- [ ] 3.3 Handle assignments without start time

## 4. Testing

- [ ] 4.1 Write unit tests for absence source mapping
- [ ] 4.2 Write unit tests for holiday source mapping
- [ ] 4.3 Write unit tests for assignment source mapping
- [ ] 4.4 Write unit tests for appointment source mapping
- [ ] 4.5 Write unit tests for read-only flags
- [ ] 4.6 Write unit tests for sort order
- [ ] 4.7 Write integration test for mixed-source day composition