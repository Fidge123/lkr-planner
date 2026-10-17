# Spec Delta

## Purpose

Mirror every project day planned in the workers' assignment calendars as one Daylite appointment linked to the project, so the schedule is visible in Daylite without inviting anyone.

## ADDED Requirements

### Requirement: One appointment per planned project day
The system SHALL keep exactly one app-managed Daylite appointment for each project and day on which at least one worker's primary calendar holds an lkr-planner assignment for that project, and none for project days without such an assignment.

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
The system SHALL create and update every app-managed appointment with type `appointment`, status `confirmed`, show-as `free`, the assignment's project as the only linked project, and the app-managed marker keyword.

#### Scenario: Attributes of a created appointment
- **WHEN** an appointment is created for project `/v1/projects/42`
- **THEN** its type is `appointment`
- **AND** its status is `confirmed`
- **AND** its show-as is `free`
- **AND** it is linked to project `/v1/projects/42`
- **AND** it carries the keyword `lkr-planner`

#### Scenario: Attributes are restored on update
- **GIVEN** an app-managed appointment whose status was changed to `tentative` in Daylite
- **WHEN** its project day is synchronized
- **THEN** its status is `confirmed` again

### Requirement: Empty invitee list
The system SHALL send an empty invitee list on every create and update of an app-managed appointment, so nobody, including the assigned workers and the requesting Daylite user, is invited.

#### Scenario: Workers are not invited
- **WHEN** an appointment is created for a project day with workers "Max Müller" and "Anna Schmidt"
- **THEN** the appointment has no invitations
- **AND** neither worker is linked to it as a contact

#### Scenario: Invitees added in Daylite are cleared
- **GIVEN** an app-managed appointment to which an invitee was added in Daylite
- **WHEN** its project day is synchronized
- **THEN** the appointment has no invitations

### Requirement: Title lists the assigned workers
The system SHALL title an app-managed appointment `<project name> (<worker names>)`, listing each worker assigned to the project on that day once, alphabetically and separated by `, `.

#### Scenario: Title for one worker
- **WHEN** only "Max Müller" is assigned to "Projekt Nord" on a day
- **THEN** the appointment title is `Projekt Nord (Max Müller)`

#### Scenario: Title for two workers
- **WHEN** "Max Müller" and "Anna Schmidt" are assigned to "Projekt Nord" on a day
- **THEN** the appointment title is `Projekt Nord (Anna Schmidt, Max Müller)`

#### Scenario: Worker display name
- **WHEN** a worker's name is put into the title
- **THEN** the name shown for that worker in the planning grid is used, which is the nickname when set and the full name otherwise

#### Scenario: Worker without a known name
- **WHEN** an assigned worker has neither a nickname nor a full name
- **THEN** the title lists that worker as `Unbenannter Kontakt`

#### Scenario: Worker with two assignments for the same project day
- **WHEN** a worker holds two assignments for "Projekt Nord" on the same day
- **THEN** the worker's name appears once in the title

### Requirement: Times follow the calendar events
The system SHALL set an app-managed appointment's start to the earliest start and its end to the latest end of the project's assignment events on that day across all workers' primary calendars, as floating times matching the calendar's wall-clock times.

#### Scenario: Times for one worker
- **WHEN** "Max Müller" is assigned to "Projekt Nord" from 08:00 to 12:00 on 2026-11-02
- **THEN** the appointment starts at 2026-11-02 08:00 and ends at 2026-11-02 12:00
- **AND** its time zone is `floating`, so it shows 08:00 to 12:00 regardless of the viewer's time zone or daylight saving time

#### Scenario: Times for workers with different slots
- **WHEN** "Max Müller" is assigned to "Projekt Nord" from 08:00 to 12:00 and "Anna Schmidt" from 12:00 to 16:00 on the same day
- **THEN** the appointment starts at 08:00 and ends at 16:00

#### Scenario: Slot re-allocation shifts the times
- **GIVEN** "Max Müller" is assigned to "Projekt Nord" from 08:00 to 16:00 on a day and the appointment covers 08:00 to 16:00
- **WHEN** a second assignment for another project is created for "Max Müller" on that day and "Projekt Nord" is re-allocated to 08:00 to 12:00
- **THEN** the "Projekt Nord" appointment is updated to 08:00 to 12:00
- **AND** an appointment for the other project is created with its re-allocated times

### Requirement: Removing a worker from a project day
The system SHALL update the appointment when a worker's last assignment for a project day is deleted or moved away, removing the worker's name from the title, and SHALL remove the appointment once no worker is assigned to that project day.

#### Scenario: One of two workers removed
- **GIVEN** "Max Müller" and "Anna Schmidt" are assigned to "Projekt Nord" on a day
- **WHEN** the assignment of "Anna Schmidt" is deleted
- **THEN** the appointment title becomes `Projekt Nord (Max Müller)`
- **AND** the times are recomputed from the remaining assignment

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
- **THEN** the appointment title becomes `Projekt Nord (Anna Schmidt)`

#### Scenario: Assignment changed to another project
- **WHEN** an assignment on a day is changed from "Projekt Nord" to "Projekt Süd"
- **THEN** the worker is removed from the "Projekt Nord" appointment for that day
- **AND** the worker is listed on the "Projekt Süd" appointment for that day

### Requirement: Synchronization after every assignment write
The system SHALL synchronize every day an assignment write touched, the day it left and the day it entered, after every successful assignment create, update, move, reorder and delete, covering all projects planned on that day.

#### Scenario: Update moves an assignment to another day
- **WHEN** an assignment is updated from 2026-11-02 to 2026-11-03
- **THEN** the appointments of both days are synchronized

#### Scenario: Unchanged appointments are not rewritten
- **WHEN** a day is synchronized and an app-managed appointment already matches its desired title, times and attributes
- **THEN** no update request is sent for that appointment

#### Scenario: Reorder triggers synchronization
- **WHEN** an assignment is reordered and the day's slots are re-allocated
- **THEN** the appointments of every project on that worker's day reflect the new times

#### Scenario: Failed calendar write
- **WHEN** an assignment write to CalDAV fails
- **THEN** no Daylite appointment is created, updated or removed

### Requirement: Only app-managed appointments are touched
The system SHALL find, update and remove only appointments carrying the `lkr-planner` keyword, and SHALL leave every other Daylite appointment untouched, even when it is linked to the same project on the same day.

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

#### Scenario: Retry reflects the current calendar
- **GIVEN** the synchronization of 2026-11-02 failed after an assignment for "Projekt Nord" was created
- **WHEN** that assignment is deleted in the calendar before the user retries
- **THEN** the retry leaves no appointment for "Projekt Nord" on 2026-11-02
