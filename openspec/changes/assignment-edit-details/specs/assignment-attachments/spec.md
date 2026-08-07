## Purpose

Let planners attach the files a job comes with -- a customer PDF, a photo of the site, a scanned form -- to the assignment itself, so the file reaches the employee in the same calendar entry instead of a separate channel.

## ADDED Requirements

### Requirement: Attach a file to an assignment
The modal SHALL let the planner attach one or more files to an assignment, and the files SHALL be stored on the calendar event so any calendar client reading the event receives them.

#### Scenario: Attach a file
- **WHEN** the planner picks a file in the modal
- **THEN** the file appears in the modal's attachment list with its name and size
- **AND** it is written to the event when the planner saves

#### Scenario: Attach several files
- **WHEN** the planner picks several files, one after another or in one selection
- **THEN** all of them appear in the attachment list
- **AND** all of them are written to the event on save

#### Scenario: Attachments are only written on save
- **WHEN** the planner attaches a file and then discards the changes
- **THEN** the event is left without that attachment

#### Scenario: Attachment keeps its name and type
- **WHEN** an attachment is written and read back
- **THEN** its file name and its content type are unchanged
- **AND** its bytes are unchanged

### Requirement: List and open attachments
The modal SHALL show the attachments an assignment already carries and SHALL let the planner open one.

#### Scenario: Existing attachments are listed
- **WHEN** the modal opens for an assignment carrying attachments
- **THEN** each attachment is listed with its file name and size

#### Scenario: Open an attachment
- **WHEN** the planner opens a listed attachment
- **THEN** the file is opened with the operating system's default application for its type

#### Scenario: Opening an attachment fails
- **WHEN** an attachment cannot be opened
- **THEN** a German error message is shown in the modal
- **AND** the modal stays open with the planner's other input intact

#### Scenario: Assignment without attachments
- **WHEN** the modal opens for an assignment carrying no attachments
- **THEN** no attachment list is shown, only the affordance to attach a file

### Requirement: Remove an attachment
The modal SHALL let the planner remove an attachment from an assignment.

#### Scenario: Remove an attachment
- **WHEN** the planner removes a listed attachment and saves
- **THEN** the event is written without it
- **AND** the assignment's other attachments are kept

#### Scenario: Remove is only applied on save
- **WHEN** the planner removes an attachment and then discards the changes
- **THEN** the event still carries it

### Requirement: Attachment size limit
The system SHALL cap the total size of an assignment's attachments and SHALL refuse an attachment that would exceed the cap, so an event stays small enough for the calendar server to accept on every rewrite.

#### Scenario: Attachment exceeds the remaining budget
- **WHEN** the planner picks a file that would push the assignment's attachments past the cap
- **THEN** the file is not added
- **AND** a German message names the cap and the size of the rejected file
- **AND** the attachments already listed are untouched

#### Scenario: Calendar server refuses the write
- **WHEN** the calendar server rejects a save because the event is too large
- **THEN** a German error message is shown in the modal
- **AND** the modal stays open with the planner's input intact
- **AND** the assignment on the server is left as it was

#### Scenario: Attachment beyond the cap already on the event
- **WHEN** an assignment already carries attachments totalling more than the cap, written elsewhere
- **THEN** they are listed and can be opened or removed
- **AND** no further attachment can be added until the total is back under the cap
