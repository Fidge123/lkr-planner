## ADDED Requirements

### Requirement: German holiday import
The system SHALL import German public holidays for week view display.

#### Scenario: Fetch Germany-wide holidays
- **WHEN** week view requests holidays for Germany
- **THEN** the service fetches holidays with `countryCode=DE` and `global=true`
- **AND** returns holiday names in German

#### Scenario: Include Mecklenburg-Vorpommern holidays
- **WHEN** week view requests holidays for Germany
- **THEN** the service also includes MV-specific holidays
- **AND** excludes other state-specific holidays

#### Scenario: Year-boundary week
- **WHEN** a week spans two years (e.g., Dec 28 - Jan 3)
- **THEN** holidays from both years are included
- **AND** holidays are correctly mapped to their respective days

#### Scenario: Cache holiday data
- **WHEN** holidays for a year have been fetched
- **THEN** subsequent requests for same year use cached data
- **AND** no additional API call is made

#### Scenario: API failure handling
- **WHEN** Nager API request fails
- **THEN** show German warning "Feiertage konnten nicht geladen werden"
- **AND** week view continues without holiday data