## 1. Backend: API Client and Module

- [ ] 1.1 Create German holiday import module (`integrations/holidays.rs`)
- [ ] 1.2 Define `Holiday` struct (date, name) and Nager API response struct
- [ ] 1.3 Implement `fetch_holidays_from_api(year)` using existing `tauri_plugin_http::reqwest`
- [ ] 1.4 Filter response: keep entries where `global == true` OR `counties` contains `"DE-MV"`; use `localName` as name

## 2. Backend: Caching

- [ ] 2.1 Add `HolidayCacheEntry` and `CachedHoliday` structs to `local_store.rs`
- [ ] 2.2 Add `holiday_cache: Vec<HolidayCacheEntry>` field to `LocalStore`
- [ ] 2.3 Implement cache lookup: current year re-fetches if older than 30 days; other years use cache if present
- [ ] 2.4 Implement cache cleanup: remove entries where `fetched_at` is older than 1 year (run on every save)

## 3. Backend: Command and Year-Boundary Handling

- [ ] 3.1 Implement `get_holidays_for_week(week_start: String)` Tauri command
- [ ] 3.2 Detect year-boundary weeks and fetch both years when needed
- [ ] 3.3 Filter fetched holidays to only those falling within the requested week before returning
- [ ] 3.4 Register command in `lib.rs`

## 4. Backend: Error Handling

- [ ] 4.1 Implement timeout handling (5s)
- [ ] 4.2 Return German error message on API failure: "Feiertage konnten nicht geladen werden"
- [ ] 4.3 Graceful degradation: frontend receives error string, continues without holiday data

## 5. Frontend: Hook and Display

- [ ] 5.1 Create `useHolidays(weekStart)` hook calling `invoke("get_holidays_for_week")`
- [ ] 5.2 Add holiday warning display in `PlanningGrid` (mirrors existing error alert pattern)
- [ ] 5.3 Update `TimetableHeader` to accept optional `holiday` prop; render holiday name below date with grey styling
- [ ] 5.4 Update `TimetableRow` to accept `holidayDates: Set<string>`; apply grey background to cells on holiday dates
- [ ] 5.5 Wire `useHolidays` into `PlanningGrid`; pass holiday data to `TimetableHeader` and `TimetableRow`

## 6. Testing

- [ ] 6.1 Write unit tests for Nager API response filtering (global, DE-MV, exclusion of other states)
- [ ] 6.2 Write unit tests for year-boundary detection and merging
- [ ] 6.3 Write unit tests for cache age logic (current year 30-day refresh, other years stable, 1-year cleanup)
- [ ] 6.4 Write unit tests for timeout and API failure behavior
