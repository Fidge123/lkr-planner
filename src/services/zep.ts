import {
  commands,
  type IcalSource,
  type ZepCalendar,
  type ZepCalendarTestResult,
  type ZepCredentialTestResult,
  type ZepCredentialsInfo,
} from "../generated/tauri";

function readZepErrorMessage(error: unknown, fallback: string): string {
  if (
    error &&
    typeof error === "object" &&
    "userMessage" in error &&
    typeof (error as { userMessage: unknown }).userMessage === "string"
  ) {
    return (error as { userMessage: string }).userMessage;
  }
  return fallback;
}

export async function loadZepCredentials(): Promise<ZepCredentialsInfo | null> {
  const result = await commands.zepLoadCredentials();
  if (result.status === "error") {
    throw new Error(
      readZepErrorMessage(
        result.error,
        "Die ZEP-Zugangsdaten konnten nicht geladen werden.",
      ),
    );
  }
  return result.data;
}

export async function testZepCredentials(
  rootUrl: string,
  username: string,
  password: string,
): Promise<ZepCredentialTestResult> {
  const result = await commands.zepTestCredentials(rootUrl, username, password);
  if (result.status === "error") {
    throw new Error(
      readZepErrorMessage(
        result.error,
        "Die ZEP-Verbindung konnte nicht getestet werden.",
      ),
    );
  }
  return result.data;
}

export async function saveZepCredentials(
  rootUrl: string,
  username: string,
  password: string,
): Promise<void> {
  const result = await commands.zepSaveCredentials(rootUrl, username, password);
  if (result.status === "error") {
    throw new Error(
      readZepErrorMessage(
        result.error,
        "Die ZEP-Zugangsdaten konnten nicht gespeichert werden.",
      ),
    );
  }
}

export async function discoverZepCalendars(): Promise<ZepCalendar[]> {
  const result = await commands.zepDiscoverCalendars();
  if (result.status === "error") {
    throw new Error(
      readZepErrorMessage(
        result.error,
        "Die ZEP-Kalender konnten nicht abgerufen werden.",
      ),
    );
  }
  return result.data;
}

export async function saveAndTestCalendar(
  dayliteContactReference: string,
  source: IcalSource,
  calendarUrl: string | null,
): Promise<ZepCalendarTestResult> {
  const result = await commands.zepSaveAndTestCalendar(
    dayliteContactReference,
    source,
    calendarUrl,
  );
  if (result.status === "error") {
    throw new Error(
      readZepErrorMessage(
        result.error,
        "Speichern und Testen fehlgeschlagen.",
      ),
    );
  }
  return result.data;
}
