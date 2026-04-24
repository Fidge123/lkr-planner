## Why

The week view needs to display German public holidays to help planners avoid scheduling on holidays. This provides essential context for planning decisions.

## What Changes

- Integrate Nager API for German public holidays
- Fetch Germany-wide (`global=true`) and Mecklenburg-Vorpommern (`DE-MV`) holidays
- Cache holiday data per year locally
- Handle year-boundary weeks (resolve holidays from both years)
- Show German warning state on API failure without breaking planning

## Capabilities

### New Capabilities
- `german-holiday-import`: Import German public holidays via Nager API

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New Rust module for holiday data fetching and caching
- APIs: Nager API (https://date.nager.at)
