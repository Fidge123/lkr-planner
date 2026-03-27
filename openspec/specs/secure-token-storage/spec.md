## Requirements

### Requirement: Secure token storage
The system SHALL store tokens in OS-level secure storage.

#### Scenario: Store token in keychain
- **GIVEN** user provides API token
- **WHEN** saving token securely
- **THEN** token is stored in macOS Keychain
- **AND** entry is retrievable by service name

#### Scenario: Retrieve token from keychain
- **GIVEN** token exists in keychain
- **WHEN** retrieving token
- **THEN** token is returned from secure storage
- **AND** no plain text token file is accessed

#### Scenario: Delete token from keychain
- **GIVEN** token exists in keychain
- **WHEN** deleting token
- **THEN** token is removed from secure storage

### Requirement: Plain text absence
The system SHALL ensure tokens are not visible in plain text files.

#### Scenario: No token in store file
- **GIVEN** token is saved via application
- **WHEN** checking local store file
- **THEN** token string does not appear in plain text

### Requirement: Frontend token access
The system SHALL restrict token access to backend only.

#### Scenario: Token fetched via Tauri command
- **GIVEN** frontend needs token for API call
- **WHEN** requesting token
- **THEN** token is retrieved via Tauri command
- **AND** token is passed to backend proxy
