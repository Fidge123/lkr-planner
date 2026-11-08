import {
  commands,
  type IcalSource,
  type ZepCalendar,
  type ZepCalendarTestResult,
  type ZepCredentialsInfo,
  type ZepCredentialTestResult,
} from "../generated/tauri";
import { callCommand } from "./command-result";

export async function loadZepCredentials(): Promise<ZepCredentialsInfo | null> {
  return callCommand(
    commands.zepLoadCredentials(),
    "Die ZEP-Zugangsdaten konnten nicht geladen werden.",
  );
}

export async function testZepCredentials(
  rootUrl: string,
  username: string,
  password: string,
): Promise<ZepCredentialTestResult> {
  return callCommand(
    commands.zepTestCredentials(rootUrl, username, password),
    "Die ZEP-Verbindung konnte nicht getestet werden.",
  );
}

export async function saveZepCredentials(
  rootUrl: string,
  username: string,
  password: string,
): Promise<void> {
  await callCommand(
    commands.zepSaveCredentials(rootUrl, username, password),
    "Die ZEP-Zugangsdaten konnten nicht gespeichert werden.",
  );
}

export async function discoverZepCalendars(): Promise<ZepCalendar[]> {
  return callCommand(
    commands.zepDiscoverCalendars(),
    "Die ZEP-Kalender konnten nicht abgerufen werden.",
  );
}

export async function saveAndTestCalendar(
  dayliteContactReference: string,
  source: IcalSource,
  calendarUrl: string | null,
): Promise<ZepCalendarTestResult> {
  return callCommand(
    commands.zepSaveAndTestCalendar(
      dayliteContactReference,
      source,
      calendarUrl,
    ),
    "Speichern und Testen fehlgeschlagen.",
  );
}
