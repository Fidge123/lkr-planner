## Context

The week view needs German public holidays to provide planning context. The Nager API provides public holiday data for Germany with state-specific filtering.

## Goals / Non-Goals

**Goals:**
- Fetch Germany-wide holidays and MV-specific holidays
- Cache year data locally to avoid repeated API calls
- Handle year-boundary weeks correctly
- Show graceful error handling without breaking UI

**Non-Goals:**
- Additional Bundesland-specific holidays beyond MV
- Holiday calendar integration beyond week view display

## Decisions

### API Integration
**Decision**: Use Nager API with country code `DE` and filter by `global` and `MV`
- Fetch all holidays for given year
- Filter client-side for global and MV states only
- Cache response in memory for same-year requests

### Year-Boundary Handling
**Decision**: Fetch holidays for both years when week spans year boundary
- If week contains days from year X and year Y
- Fetch holidays for both years and merge

### Caching Strategy
**Decision**: In-memory cache with year as key
- Avoids repeated API calls within session
- Cache invalidates on year change or explicit refresh

## Risks / Trade-offs

- **Risk**: Nager API unavailability
  - **Mitigation**: Show German warning "Feiertage konnten nicht geladen werden" and continue without holidays

- **Risk**: Rate limiting
  - **Mitigation**: Cache aggressively, add retry with backoff
