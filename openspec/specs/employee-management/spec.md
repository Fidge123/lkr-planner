## Requirements

### Requirement: Employee Roster Sync
The system SHALL populate the active employee roster using Daylite contacts categorized as "Monteur".

#### Scenario: Load Employees
- **GIVEN** Daylite contains contacts with the "Monteur" category
- **WHEN** the application loads the employee list
- **THEN** those specific contacts are mapped to Employee domain types

### Requirement: Employee iCal Integration
The system SHALL extract and handle iCal URLs stored within Daylite contact records for schedule parsing.

#### Scenario: Read iCal Configuration
- **GIVEN** an employee mapped from a Daylite contact
- **WHEN** accessing their calendar source
- **THEN** the system utilizes the URL fields associated with that contact

### Requirement: Daylite Write Operations
The system SHALL support updating employee metadata back to Daylite.

#### Scenario: Update Employee Info
- **GIVEN** an employee mapped from Daylite
- **WHEN** modifying the employee properties in the frontend
- **THEN** a write command updates the corresponding Daylite contact record
