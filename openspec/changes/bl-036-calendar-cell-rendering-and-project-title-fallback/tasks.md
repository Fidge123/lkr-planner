## 1. Title Fallback Logic

- [ ] 1.1 Implement `getProjectTitle(assignment): string` function
- [ ] 1.2 Implement fallback check: custom name → Planradar → Daylite company (single) → Daylite project
- [ ] 1.3 Add helper to check linked company count

## 2. Cell Item Rendering

- [ ] 2.1 Create `CalendarCellItemRow` component
- [ ] 2.2 Implement read-only row component for absence/holiday/appointment
- [ ] 2.3 Implement clickable row component for assignment
- [ ] 2.4 Add time display formatting

## 3. Integration

- [ ] 3.1 Connect renderer to BL-035 composition output
- [ ] 3.2 Wire click handler to open edit modal
- [ ] 3.3 Add all-day indicator for absence/holiday

## 4. Testing

- [ ] 4.1 Write unit tests for title fallback with custom name
- [ ] 4.2 Write unit tests for title fallback with Planradar name
- [ ] 4.3 Write unit tests for title fallback with single Daylite company
- [ ] 4.4 Write unit tests for title fallback with Daylite project
- [ ] 4.5 Write UI tests for all item types in one cell
- [ ] 4.6 Write UI tests for read-only enforcement