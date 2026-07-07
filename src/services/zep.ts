import {
  commands,
  type IcalSource,
  type ZepCalendar,
  type ZepCalendarTestResult,
  type ZepCredentialsInfo,
  type ZepCredentialTestResult,
} from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";

export async function loadZepCredentials(): Promise<ZepCredentialsInfo | null> {
  return unwrapCommandResult(
    await commands.zepLoadCredentials(),
    "Die ZEP-Zugangsdaten konnten nicht geladen werden.",
  );
}

export async function testZepCredentials(
  rootUrl: string,
  username: string,
  password: string,
): Promise<ZepCredentialTestResult> {
  return unwrapCommandResult(
    await commands.zepTestCredentials(rootUrl, username, password),
    "Die ZEP-Verbindung konnte nicht getestet werden.",
  );
}

export async function saveZepCredentials(
  rootUrl: string,
  username: string,
  password: string,
): Promise<void> {
  unwrapCommandResult(
    await commands.zepSaveCredentials(rootUrl, username, password),
    "Die ZEP-Zugangsdaten konnten nicht gespeichert werden.",
  );
}

export async function discoverZepCalendars(): Promise<ZepCalendar[]> {
  return unwrapCommandResult(
    await commands.zepDiscoverCalendars(),
    "Die ZEP-Kalender konnten nicht abgerufen werden.",
  );
}

export async function saveAndTestCalendar(
  dayliteContactReference: string,
  source: IcalSource,
  calendarUrl: string | null,
): Promise<ZepCalendarTestResult> {
  return unwrapCommandResult(
    await commands.zepSaveAndTestCalendar(
      dayliteContactReference,
      source,
      calendarUrl,
    ),
    "Speichern und Testen fehlgeschlagen.",
  );
}
