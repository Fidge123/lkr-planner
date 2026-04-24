## ADDED Requirements

### Requirement: German holiday import
The system SHALL import German public holidays for week view display.

#### Scenario: Fetch Germany-wide holidays
- **WHEN** week view requests holidays for Germany
- **THEN** the service fetches holidays with `countryCode=DE`
- **AND** returns holiday entries where `global == true` using `localName` as the German name

#### Scenario: Include Mecklenburg-Vorpommern holidays
- **WHEN** week view requests holidays for Germany
- **THEN** the service also includes entries where `counties` contains `"DE-MV"`
- **AND** excludes entries from other German states

#### Scenario: Year-boundary week
- **WHEN** a week spans two years (e.g., Dec 28 - Jan 3)
- **THEN** holidays from both years are fetched and merged
- **AND** holidays are correctly mapped to their respective days

#### Scenario: Cache current year with monthly refresh
- **WHEN** holidays for the current year have been fetched
- **THEN** subsequent requests within 30 days use cached data
- **AND** no additional API call is made

#### Scenario: Refresh stale current-year cache
- **WHEN** the cached entry for the current year is older than 30 days
- **THEN** the service re-fetches from the Nager API
- **AND** updates the cache with fresh data and a new `fetched_at` date

#### Scenario: Cache other years indefinitely
- **WHEN** holidays for a past or future year have been fetched
- **THEN** subsequent requests for that year always use cached data
- **AND** no re-fetch occurs regardless of cache age

#### Scenario: Clean up old cache entries
- **WHEN** the holiday cache is saved
- **THEN** any entry whose `fetched_at` is older than 1 year is removed

#### Scenario: API failure handling
- **WHEN** Nager API request fails
- **THEN** the command returns a German error message "Feiertage konnten nicht geladen werden"
- **AND** the week view continues to display without holiday data

#### Scenario: Holiday name in column header
- **WHEN** a week day is a public holiday
- **THEN** the column header shows the weekday and date on the first line
- **AND** the German holiday name on the second line
- **AND** the header is styled in grey

#### Scenario: Grey out holiday column
- **WHEN** a week day is a public holiday
- **THEN** all employee cells in that column are displayed with a grey background
