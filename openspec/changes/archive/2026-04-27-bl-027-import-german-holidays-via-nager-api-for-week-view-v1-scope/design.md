## Context

The week view needs German public holidays to provide planning context. The Nager API provides public holiday data for Germany with state-specific filtering.

## Goals / Non-Goals

**Goals:**
- Fetch Germany-wide holidays and MV-specific holidays
- Cache year data on disk to avoid repeated API calls
- Handle year-boundary weeks correctly
- Show graceful error handling without breaking UI
- Display holiday name in column header below the date
- Grey out holiday columns for all employees

**Non-Goals:**
- Additional Bundesland-specific holidays beyond MV
- Holiday calendar integration beyond week view display

## Decisions

### API Integration
**Decision**: Use Nager API with country code `DE`, filter client-side by `global` and `DE-MV` county
- Endpoint: `GET https://date.nager.at/api/v3/PublicHolidays/{year}/DE`
- Use `localName` field for German holiday name display
- Keep holiday if `global == true` OR `counties` contains `"DE-MV"`

### Year-Boundary Handling
**Decision**: Fetch holidays for both years when week spans year boundary
- If week contains days from year X and year Y
- Fetch holidays for both years and merge

### Caching Strategy
**Decision**: Disk cache via LocalStore with per-year entries and age-based refresh
- Cache shape: `Vec<HolidayCacheEntry>` added to `LocalStore`
  - `year: i32`, `fetched_at: String` (yyyy-MM-dd), `holidays: Vec<CachedHoliday>`
  - `CachedHoliday`: `date: String` (yyyy-MM-dd), `name: String` (German)
- **Current year**: re-fetch if `fetched_at` is older than 30 days (holidays can be corrected)
- **Other years**: re-fetch only if entry is absent (past years are stable)
- **Cleanup**: on every save, remove cache entries where `fetched_at` is older than 1 year

### Frontend Display
**Decision**: Holiday name shown below the date in the column header; holiday columns greyed out
- `TimetableHeader` receives an optional `holiday` prop
- When present: render date on first line, holiday name (German) on second line; apply grey styling to header
- `TimetableRow` receives a `holidayDates: Set<string>` prop (yyyy-MM-dd strings)
- Cells whose date is in `holidayDates` receive grey background styling
- `PlanningGrid` owns the `useHolidays(weekStart)` hook and passes data down

### Tauri Command
**Decision**: Single command returning holidays for a given week
- Signature: `get_holidays_for_week(week_start: String) -> Result<Vec<Holiday>, String>`
- `Holiday`: `date: String` (yyyy-MM-dd), `name: String`
- Backend handles year-boundary, caching, and filtering before returning
- Frontend receives only the holidays that fall within the requested week

## Risks / Trade-offs

- **Risk**: Nager API unavailability
  - **Mitigation**: Show German warning "Feiertage konnten nicht geladen werden" and continue without holidays

- **Risk**: Rate limiting
  - **Mitigation**: Disk cache with monthly refresh for current year; past years cached indefinitely
