## REMOVED Requirements

### Requirement: Data minimization
**Reason**: Redacting URLs, paths and e-mail addresses removed the details needed to diagnose errors remotely, and telemetry is opt-in under a text that names the transmitted data.
**Migration**: Replaced by the Secret removal requirement, which removes only credentials.

## ADDED Requirements

### Requirement: Secret removal
The system SHALL remove credentials from every transmitted free-text field and SHALL transmit all other error details unchanged.

#### Scenario: Error details retained
- **GIVEN** a technical error message that names a URL, a file path, or an e-mail address
- **WHEN** the error event is built
- **THEN** the URL, the file path, and the e-mail address are transmitted unchanged

#### Scenario: Credential key-value pairs removed
- **GIVEN** a technical error message containing a token, password, secret, or key as a key-value pair in plain text, in a URL query string, or in JSON
- **WHEN** the error event is built
- **THEN** the value is redacted
- **AND** the key and the surrounding message are retained

#### Scenario: Authorization values removed
- **GIVEN** a technical error message containing a bearer or basic authorization value
- **WHEN** the error event is built
- **THEN** the value is redacted

#### Scenario: URL credentials removed
- **GIVEN** a technical error message containing a URL with a user name and password
- **WHEN** the error event is built
- **THEN** the user name and the password are redacted
- **AND** the rest of the URL is retained

#### Scenario: Truncated credential removed
- **GIVEN** a technical error message that ends inside a credential value
- **WHEN** the error event is built
- **THEN** the message is redacted from the start of that value to its end

#### Scenario: Command message transmitted
- **GIVEN** a backend command fails with only a user-facing message
- **WHEN** the error event is built
- **THEN** the user-facing message is transmitted with credentials removed

## MODIFIED Requirements

### Requirement: Consent visibility
The system SHALL expose the telemetry preference in the settings dialog with a German description of what is collected.

#### Scenario: Telemetry setting reachable
- **WHEN** the user opens the settings dialog
- **THEN** a section for diagnostics is listed
- **AND** it contains a control to enable or disable telemetry

#### Scenario: Collection described in German
- **WHEN** the diagnostics section is shown
- **THEN** it states in German that error and performance data is transmitted
- **AND** it states that error reports contain technical details and can contain project or employee names
- **AND** it states that the IP address is transmitted
- **AND** it states that passwords, keys, and tokens are never transmitted

#### Scenario: Current state reflected
- **GIVEN** telemetry is enabled
- **WHEN** the user reopens the settings dialog
- **THEN** the control shows telemetry as enabled
