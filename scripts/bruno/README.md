# LKR Planner Bruno collection

This [Bruno](https://www.usebruno.com/) collection replays the HTTP calls the LKR Planner backend makes to Daylite, ZEP CalDAV and the public holiday API.

## Setup

Open `scripts/bruno` as a collection in Bruno.
The scripts load `lib/keychain.js`, Node modules and files outside the collection, so switch the collection to the Developer Mode JavaScript sandbox.
CalDAV requests use the custom methods `PROPFIND` and `REPORT`, which need a Bruno version that supports custom HTTP methods.

## Environments

Both environments share the same requests and differ only in where credentials and API endpoints come from.
Set `zepCalendarUrl` to a calendar collection with a trailing slash, for example one returned by "List calendars".

### Manual

Enter `dayliteBaseUrl` and `zepCaldavRootUrl` as plain variables and `dayliteRefreshToken`, `zepUsername` and `zepPassword` as secrets in the environment.
Run "Daylite / Refresh token" first, it stores the new access and refresh token in the environment.
Daylite refresh tokens are single use, so do not use the token pair LKR Planner currently holds or the app loses its Daylite connection.

### Keychain

Reads the credentials LKR Planner stored in the macOS keychain via the `security` command.
Reads the Daylite base URL and the ZEP CalDAV root URL from the app's `~/Library/Application Support/de.richter-labor.planner/local-store.json`.
macOS asks once per item whether Bruno may read it, choose "Always Allow" to avoid repeated prompts.
The Daylite access token is used as is and never rotated by the collection, "Refresh token" refuses to run in this environment.
When the token has expired, trigger any Daylite action in LKR Planner so the app rotates it, then retry.

## Requests

- Daylite: refresh token, project search, overdue projects, single project, project categories, planning contacts, single contact and the contact iCal URL update.
- ZEP CalDAV: calendar discovery, calendar probe, events in a date range, and reading, creating, updating and deleting a single event.
- Holidays: German public holidays for a year from date.nager.at.

"Events in range" always queries the current week, starting Monday 00:00 local time.
"Create event" stores the generated UID in `eventUid`, so the following event requests target the event it created.
"Update contact iCal URLs" replaces every URL of the contact, so copy the contact's other URLs into the body first.
