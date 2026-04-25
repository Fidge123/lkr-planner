## 1. Backend: API Client and Module

- [x] 1.1 Create German holiday import module (`integrations/holidays.rs`)
- [x] 1.2 Define `Holiday` struct (date, name) and Nager API response struct
- [x] 1.3 Write unit tests for Nager API response filtering (global, DE-MV, exclusion of other states)
- [x] 1.4 Implement `fetch_holidays_from_api(year)` using existing `tauri_plugin_http::reqwest`
- [x] 1.5 Filter response: keep entries where `global == true` OR `counties` contains `"DE-MV"`; use `localName` as name

## 2. Backend: Caching

- [x] 2.1 Add `HolidayCacheEntry` and `CachedHoliday` structs to `local_store.rs`
- [x] 2.2 Add `holiday_cache: Vec<HolidayCacheEntry>` field to `LocalStore`
- [x] 2.3 Write unit tests for cache age logic (30-day refresh, 1-year cleanup)
- [x] 2.4 Implement cache lookup: re-fetches if older than 30 days
- [x] 2.5 Implement cache cleanup: remove entries where `fetched_at` is older than 1 year (run on every save)

## 3. Backend: Command and Year-Boundary Handling

- [x] 3.1 Write unit tests for year-boundary detection and merging
- [x] 3.2 Implement `get_holidays_for_week(week_start: String)` Tauri command
- [x] 3.3 Detect year-boundary weeks and fetch both years when needed
- [x] 3.4 Filter fetched holidays to only those falling within the requested week before returning
- [x] 3.5 Register command in `lib.rs`

## 4. Backend: Error Handling

- [x] 4.1 Write unit tests for timeout and API failure behavior
- [x] 4.2 Implement timeout handling (5s)
- [x] 4.3 Return German error message on API failure: "Feiertage konnten nicht geladen werden"
- [x] 4.4 Graceful degradation: frontend receives error string, continues without holiday data

## 5. Frontend: Hook and Display

- [x] 5.1 Create `useHolidays(weekStart)` hook calling `invoke("get_holidays_for_week")`
- [x] 5.2 Add holiday warning display in `PlanningGrid` (mirrors existing error alert pattern)
- [x] 5.3 Update `TimetableHeader` to accept optional `holiday` prop; render holiday name below date with grey styling
- [x] 5.4 Update `TimetableRow` to accept `holidayDates: Set<string>`; apply grey background to cells on holiday dates
- [x] 5.5 Wire `useHolidays` into `PlanningGrid`; pass holiday data to `TimetableHeader` and `TimetableRow`
