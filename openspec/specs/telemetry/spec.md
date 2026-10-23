## Purpose

Make failures and slow operations in a distributed desktop app observable to the maintainer by sending opt-in error and performance events to PostHog with enough detail to diagnose them remotely, without ever transmitting credentials.

## Requirements

### Requirement: Opt-in consent gate
The system SHALL treat telemetry as disabled unless the user has explicitly enabled it, and SHALL neither record nor transmit any event while telemetry is disabled.

#### Scenario: Telemetry off on first launch
- **GIVEN** the application is started with no stored telemetry preference
- **WHEN** an error occurs or an operation completes
- **THEN** no event is recorded and no request is sent to the telemetry endpoint

#### Scenario: Existing installation keeps telemetry off
- **GIVEN** a stored configuration written before telemetry existed
- **WHEN** the configuration is loaded
- **THEN** telemetry is reported as disabled
- **AND** no event is recorded until the user enables it

#### Scenario: User enables telemetry
- **GIVEN** telemetry is disabled
- **WHEN** the user enables telemetry in the settings dialog
- **THEN** the preference is persisted
- **AND** subsequent errors and completed operations are recorded

#### Scenario: User disables telemetry
- **GIVEN** telemetry is enabled and events are pending delivery
- **WHEN** the user disables telemetry
- **THEN** the preference is persisted
- **AND** all pending events are discarded without being sent
- **AND** no further events are recorded

### Requirement: Anonymous install identity
The system SHALL identify events by a randomly generated install identifier that contains no user, device, or account information, and SHALL keep that identifier stable across restarts of the same installation.

#### Scenario: Identifier generated on first activation
- **GIVEN** telemetry is enabled and no install identifier exists
- **WHEN** the first event is recorded
- **THEN** a random identifier is generated and persisted
- **AND** the event carries that identifier

#### Scenario: Identifier stable across restarts
- **GIVEN** an install identifier has been persisted
- **WHEN** the application is restarted and an event is recorded
- **THEN** the event carries the same identifier

#### Scenario: Identifier is not derived from user data
- **WHEN** an install identifier is generated
- **THEN** it is a random value
- **AND** it is not derived from user name, host name, e-mail address, account data, or hardware identifiers

### Requirement: Error events
The system SHALL record an error event when a backend operation fails, when an outbound request to an integration fails, and when the user interface encounters an uncaught error.

#### Scenario: Backend command fails
- **GIVEN** telemetry is enabled
- **WHEN** a backend command returns a structured error
- **THEN** an error event is recorded carrying the operation name, the integration name, the error code, and a sanitized technical message

#### Scenario: Outbound integration request fails
- **GIVEN** telemetry is enabled
- **WHEN** a request to Daylite, the ZEP CalDAV server, or the holiday API fails or returns an unsuccessful status
- **THEN** an error event is recorded carrying the integration name, the request kind, and the response status when one was received

#### Scenario: Uncaught frontend error
- **GIVEN** telemetry is enabled
- **WHEN** a rendering error or an unhandled promise rejection occurs in the user interface
- **THEN** an error event is recorded carrying the error name, a sanitized message, and the component or handler context

#### Scenario: Error is still shown to the user
- **WHEN** an error event is recorded
- **THEN** the German user-facing error message is displayed unchanged
- **AND** the displayed message does not depend on the outcome of telemetry recording

### Requirement: Performance events
The system SHALL record the measured duration of backend commands, outbound integration requests, and application startup, and SHALL record background work under operation names distinct from the waits a user sees.

#### Scenario: Backend command duration
- **GIVEN** telemetry is enabled
- **WHEN** a backend command completes, successfully or with an error
- **THEN** a performance event is recorded carrying the operation name, the duration in milliseconds, and whether the operation succeeded

#### Scenario: Outbound request duration
- **GIVEN** telemetry is enabled
- **WHEN** a request to an integration completes
- **THEN** a performance event is recorded carrying the integration name, the request kind, the duration in milliseconds, and the response status when one was received

#### Scenario: Background week loads are distinguishable
- **GIVEN** telemetry is enabled
- **WHEN** a week is loaded to prefetch it, or to refresh the rows of an edit
- **THEN** its performance event carries the operation `prefetch_week_events` or `refresh_week_events` instead of `load_week_events`

#### Scenario: Project resolution is recorded on its own
- **GIVEN** telemetry is enabled
- **WHEN** the Daylite projects of a week are resolved
- **THEN** a performance event with the operation `resolve_week_projects` is recorded
- **AND** each project lookup is recorded as an outbound Daylite request

#### Scenario: Startup duration
- **GIVEN** telemetry is enabled
- **WHEN** the application finishes starting up
- **THEN** a performance event is recorded carrying the duration from process start to the ready state

### Requirement: Event context
The system SHALL attach the application version, the operating system name and version, and the install identifier to every transmitted event.

#### Scenario: Context present on every event
- **GIVEN** telemetry is enabled
- **WHEN** any event is transmitted
- **THEN** it carries the application version, the operating system name and version, and the install identifier

#### Scenario: Events distinguishable per release
- **GIVEN** events recorded from two different application versions
- **WHEN** the events are inspected
- **THEN** each event's application version identifies the release it came from

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

### Requirement: Batched delivery
The system SHALL buffer recorded events and transmit them in batches to the telemetry endpoint, in the background, without blocking the operation that produced them.

#### Scenario: Events sent in batches
- **GIVEN** telemetry is enabled and several events have been recorded
- **WHEN** the flush interval elapses or the batch size is reached
- **THEN** the buffered events are transmitted in a single request
- **AND** transmitted events are removed from the buffer

#### Scenario: Recording does not block the caller
- **WHEN** an event is recorded during a user-facing operation
- **THEN** the operation returns without waiting for the event to be transmitted

#### Scenario: Pending events flushed on shutdown
- **GIVEN** telemetry is enabled and events are buffered
- **WHEN** the application shuts down
- **THEN** a final delivery attempt is made within a bounded time
- **AND** shutdown is not delayed beyond that bound if delivery does not complete

### Requirement: Failure isolation
The system SHALL contain all telemetry failures within the telemetry path, so that no telemetry condition degrades or interrupts the planning workflow.

#### Scenario: Telemetry endpoint unreachable
- **GIVEN** telemetry is enabled and the endpoint cannot be reached
- **WHEN** a delivery attempt fails
- **THEN** the failure is not surfaced to the user
- **AND** the planning workflow continues unaffected

#### Scenario: Buffer bounded when offline
- **GIVEN** telemetry is enabled and delivery has been failing
- **WHEN** the number of buffered events reaches the configured limit
- **THEN** the oldest events are dropped
- **AND** memory use stays bounded

#### Scenario: No telemetry key configured
- **GIVEN** the application was built without a telemetry project key
- **WHEN** telemetry is enabled and an event is recorded
- **THEN** no request is sent to any endpoint
- **AND** the application behaves as if telemetry were disabled

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
