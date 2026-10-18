# Spec Delta

## Purpose

Record in each project's Daylite timeline, next to its notes and emails, when the team works on the project, by mirroring every project day planned in lkr-planner as one Daylite appointment linked to the project and its workers without inviting anyone.

## ADDED Requirements

### Requirement: Planned work appears with the project and its workers
The system SHALL link every appointment it creates to the assignment's Daylite project and to the Daylite contact of every worker assigned to that project on that day, so the appointment appears in the project's timeline and with each worker's contact.

#### Scenario: Appointment shows in the project timeline
- **WHEN** "Max Müller" is assigned to "Projekt Nord" on 2026-11-02 through lkr-planner
- **THEN** the timeline of "Projekt Nord" in Daylite shows the appointment `Einsatz 02.11.2026`
- **AND** that appointment is linked to the contact of "Max Müller"

### Requirement: One appointment per planned project day
The system SHALL keep exactly one app-managed Daylite appointment for each project and day on which at least one planning contact's primary calendar holds an lkr-planner assignment for that project, and none for project days without such an assignment.
Planning contacts are the Daylite contacts the planning grid lists, Monteur and Test contacts alike.

#### Scenario: Test contacts count like Monteur contacts
- **WHEN** a Test contact is assigned to "Projekt Nord" on 2026-11-02
- **THEN** the appointment for "Projekt Nord" on 2026-11-02 is linked to that contact

#### Scenario: Calendar of a former planning contact
- **GIVEN** a primary calendar is still configured for a contact that is no longer a Monteur or Test contact
- **WHEN** a project day is synchronized
- **THEN** that calendar is not read

#### Scenario: First worker assigned to a project on a day
- **GIVEN** no worker is assigned to project "Projekt Nord" on 2026-11-02
- **WHEN** an assignment for "Projekt Nord" on 2026-11-02 is created for worker "Max Müller"
- **THEN** one Daylite appointment is created for "Projekt Nord" on 2026-11-02

#### Scenario: Second worker assigned to the same project and day
- **GIVEN** a Daylite appointment exists for "Projekt Nord" on 2026-11-02 for worker "Max Müller"
- **WHEN** an assignment for "Projekt Nord" on 2026-11-02 is created for worker "Anna Schmidt"
- **THEN** the existing appointment is updated
- **AND** no second appointment is created

#### Scenario: Same project on different days
- **WHEN** a worker is assigned to "Projekt Nord" on 2026-11-02 and on 2026-11-03
- **THEN** each day has its own Daylite appointment

#### Scenario: Duplicate app-managed appointments for one project day
- **GIVEN** two app-managed Daylite appointments exist for the same project and day
- **WHEN** that project day is synchronized
- **THEN** the appointment with the lowest Daylite ID is kept and updated
- **AND** the other is removed

### Requirement: Fixed appointment attributes
The system SHALL create and update every app-managed appointment with the title `Einsatz <date>`, its day formatted `dd.MM.yyyy`, the floating times 08:00 to 16:00 on that day, type `appointment`, status `confirmed`, show-as `free`, the assignment's project as the only linked project, and the app-managed marker keyword.

#### Scenario: Attributes of a created appointment
- **WHEN** an appointment is created for project `/v1/projects/42` on 2026-11-02
- **THEN** its title is `Einsatz 02.11.2026`
- **AND** it starts at 2026-11-02 08:00 and ends at 2026-11-02 16:00
- **AND** its time zone is `floating`, so it shows 08:00 to 16:00 regardless of the viewer's time zone or daylight saving time
- **AND** its type is `appointment`
- **AND** its status is `confirmed`
- **AND** its show-as is `free`
- **AND** it is linked to project `/v1/projects/42`
- **AND** it carries the keyword `lkr-planner`

#### Scenario: Times do not follow the assignment events
- **WHEN** "Max Müller" is assigned to "Projekt Nord" from 12:00 to 16:00 on 2026-11-02
- **THEN** the appointment for "Projekt Nord" on 2026-11-02 runs from 08:00 to 16:00

#### Scenario: Slot re-allocation leaves the appointment unchanged
- **GIVEN** "Max Müller" is assigned to "Projekt Nord" on 2026-11-02 and its appointment exists
- **WHEN** a second assignment for "Projekt Süd" is created for "Max Müller" on that day and "Projekt Nord" is re-allocated to 08:00 to 12:00
- **THEN** the "Projekt Nord" appointment is not updated
- **AND** an appointment for "Projekt Süd" is created from 08:00 to 16:00

#### Scenario: Attributes are restored on update
- **GIVEN** an app-managed appointment whose status was changed to `tentative` and whose title was changed in Daylite
- **WHEN** its project day is synchronized
- **THEN** its status is `confirmed` again
- **AND** its title is `Einsatz <date>` again

### Requirement: Empty invitee list
The system SHALL send an empty invitee list on every create and update of an app-managed appointment, so nobody, including the assigned workers and the requesting Daylite user, is invited.

#### Scenario: Workers are linked but not invited
- **WHEN** an appointment is created for a project day with workers "Max Müller" and "Anna Schmidt"
- **THEN** the appointment has no invitations
- **AND** neither worker receives an invitation
- **AND** both workers are linked to it as contacts

#### Scenario: Invitees added in Daylite are cleared
- **GIVEN** an app-managed appointment to which an invitee was added in Daylite
- **WHEN** its project day is synchronized
- **THEN** the appointment has no invitations

### Requirement: Linked contacts are the assigned workers
The system SHALL link an app-managed appointment to exactly the Daylite contacts of the workers assigned to its project on its day, each contact once, and to no other contact.

#### Scenario: Contacts for one worker
- **WHEN** only "Max Müller" is assigned to "Projekt Nord" on a day
- **THEN** the appointment is linked to the contact of "Max Müller" only

#### Scenario: Contacts for two workers
- **WHEN** "Max Müller" and "Anna Schmidt" are assigned to "Projekt Nord" on a day
- **THEN** the appointment is linked to the contacts of "Max Müller" and "Anna Schmidt"

#### Scenario: Worker with two assignments for the same project day
- **WHEN** a worker holds two assignments for "Projekt Nord" on the same day
- **THEN** the worker's contact is linked once

#### Scenario: Contacts added in Daylite are removed
- **GIVEN** a contact other than the assigned workers was linked to an app-managed appointment in Daylite
- **WHEN** its project day is synchronized
- **THEN** only the contacts of the assigned workers remain linked

### Requirement: Removing a worker from a project day
The system SHALL update the appointment when a worker's last assignment for a project day is deleted or moved away, unlinking the worker's contact, and SHALL remove the appointment once no worker is assigned to that project day.

#### Scenario: One of two workers removed
- **GIVEN** "Max Müller" and "Anna Schmidt" are assigned to "Projekt Nord" on a day
- **WHEN** the assignment of "Anna Schmidt" is deleted
- **THEN** the appointment is linked to the contact of "Max Müller" only

#### Scenario: Last worker removed
- **GIVEN** only "Max Müller" is assigned to "Projekt Nord" on a day
- **WHEN** that assignment is deleted
- **THEN** the appointment is removed from Daylite

#### Scenario: Assignment moved to another day
- **WHEN** the only assignment for "Projekt Nord" on 2026-11-02 is moved to 2026-11-03
- **THEN** the appointment for 2026-11-02 is removed
- **AND** an appointment for 2026-11-03 exists

#### Scenario: Assignment moved to another worker
- **WHEN** the assignment of "Max Müller" for "Projekt Nord" on a day is moved to "Anna Schmidt" on the same day
- **THEN** the appointment is linked to the contact of "Anna Schmidt"
- **AND** it is no longer linked to the contact of "Max Müller"

#### Scenario: Assignment changed to another project
- **WHEN** an assignment of "Max Müller" on a day is changed from "Projekt Nord" to "Projekt Süd"
- **THEN** the contact of "Max Müller" is unlinked from the "Projekt Nord" appointment for that day
- **AND** it is linked to the "Projekt Süd" appointment for that day

### Requirement: Synchronization after assignment writes through the app
The system SHALL synchronize the appointments after every successful assignment create, update, move and delete made through lkr-planner, for the day the write left and the day it entered, covering all projects planned on those days.
Reordering an assignment within its day changes nothing an appointment holds and SHALL NOT trigger synchronization, and neither SHALL calendar entries the app does not write.

#### Scenario: Update moves an assignment to another day
- **WHEN** an assignment is updated from 2026-11-02 to 2026-11-03
- **THEN** the appointments of both days reflect the calendar after the update

#### Scenario: Entry the app did not write
- **GIVEN** a calendar entry linked to "Projekt Nord" on 2026-11-02 that was created outside lkr-planner or before this feature existed
- **WHEN** the week containing 2026-11-02 is loaded
- **THEN** no Daylite appointment is created or changed

#### Scenario: Entry the app did not write is covered once its day is written
- **GIVEN** a calendar entry linked to "Projekt Nord" on 2026-11-02 that has no Daylite appointment
- **WHEN** any assignment on 2026-11-02 is created, updated, moved or deleted through lkr-planner
- **THEN** "Projekt Nord" has an appointment for 2026-11-02 reflecting all its assignments of that day

#### Scenario: Reorder leaves Daylite unchanged
- **WHEN** an assignment is reordered within its day and the day's slots are re-allocated
- **THEN** no Daylite appointment is created, updated or removed

#### Scenario: Failed calendar write
- **WHEN** an assignment write to CalDAV fails
- **THEN** no Daylite appointment is created, updated or removed

### Requirement: Only app-managed appointments are touched
The system SHALL treat an appointment as app-managed only when it carries the `lkr-planner` keyword, and SHALL find, update and remove only such appointments, and SHALL leave every other Daylite appointment untouched, even when it is linked to the same project on the same day.

#### Scenario: Manual appointment on the same project day
- **GIVEN** a Daylite appointment without the `lkr-planner` keyword is linked to "Projekt Nord" on a day
- **WHEN** the last assignment for "Projekt Nord" on that day is deleted
- **THEN** that appointment is neither updated nor removed

### Requirement: Synchronization failures do not fail the assignment write
The system SHALL treat Daylite synchronization as a follow-up to the calendar write: a synchronization failure SHALL NOT fail, roll back or automatically retry the assignment write, SHALL be recorded in telemetry, and SHALL be reported to the user.

#### Scenario: Daylite unreachable after a create
- **WHEN** an assignment is created in CalDAV and the Daylite appointment request then fails
- **THEN** the create command still reports success
- **AND** the failure is recorded as a telemetry error
- **AND** the user is shown the synchronization error

#### Scenario: Later write converges
- **GIVEN** a project day whose appointment could not be synchronized earlier
- **WHEN** any assignment write later touches that project day
- **THEN** the appointment reflects the current calendar state

#### Scenario: Worker calendar unreadable during synchronization
- **WHEN** a worker's primary calendar cannot be read while a project day is being synchronized
- **THEN** no appointment for that day is created, updated or removed
- **AND** the failure is recorded as a telemetry error
- **AND** the day counts as failed

#### Scenario: Worker calendar that does not exist
- **WHEN** the calendar server reports a worker's configured primary calendar as not found while a project day is being synchronized
- **THEN** that worker counts as having no assignments on that day
- **AND** the day is synchronized from the other workers' calendars

### Requirement: Synchronization failures are shown to the user
The system SHALL show one German error alert above the planning grid while at least one day's synchronization has failed and has not succeeded since, naming every such day and offering an `Erneut versuchen` button.

#### Scenario: Failure shows the alert
- **WHEN** the synchronization of 2026-11-02 fails
- **THEN** an error alert reading `Daylite-Termine für folgende Tage konnten nicht aktualisiert werden: 02.11.2026` is shown
- **AND** it carries an `Erneut versuchen` button

#### Scenario: Several failed days share one alert
- **WHEN** the synchronizations of 2026-11-03 and 2026-11-02 fail
- **THEN** one alert lists `02.11.2026, 03.11.2026` in ascending order

#### Scenario: A later successful synchronization clears the day
- **GIVEN** the synchronization of 2026-11-02 failed
- **WHEN** a later assignment write synchronizes 2026-11-02 successfully
- **THEN** 2026-11-02 is no longer listed
- **AND** the alert is hidden once no failed day remains

#### Scenario: Failed days are not remembered across restarts
- **GIVEN** failed days are listed in the alert
- **WHEN** the app is restarted
- **THEN** no synchronization alert is shown

### Requirement: Retry failed synchronization
The system SHALL let the user retry from the alert, re-running the whole-day synchronization for every listed day against the calendar state at the time of the retry.

#### Scenario: Retry succeeds
- **GIVEN** the alert lists 2026-11-02 and 2026-11-03
- **WHEN** the user clicks `Erneut versuchen` and both days synchronize successfully
- **THEN** the alert disappears

#### Scenario: Retry partially fails
- **GIVEN** the alert lists 2026-11-02 and 2026-11-03
- **WHEN** the retry succeeds for 2026-11-02 and fails for 2026-11-03
- **THEN** the alert lists only `03.11.2026`

#### Scenario: Retry in progress
- **WHEN** a retry is running
- **THEN** the button is disabled and reads `Wird wiederholt...`
- **AND** further clicks do not start a second retry

#### Scenario: Retry after a long outage
- **GIVEN** the alert lists many days
- **WHEN** the user clicks `Erneut versuchen`
- **THEN** at most two days are synchronized at the same time

#### Scenario: Retry reflects the current calendar
- **GIVEN** the synchronization of 2026-11-02 failed after an assignment for "Projekt Nord" was created
- **WHEN** that assignment is deleted in the calendar before the user retries
- **THEN** the retry leaves no appointment for "Projekt Nord" on 2026-11-02
