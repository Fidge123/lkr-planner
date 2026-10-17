import type { EmployeeSetting } from "../generated/tauri";

export interface StartupSync {
  /** Null when the local store could not be read. */
  readSettings: () => Promise<EmployeeSetting[] | null>;
  syncContacts: () => Promise<void>;
  reloadWeek: () => void;
  isCancelled: () => boolean;
}

/**
 * The contact sync reconciles calendar URLs from Daylite into the local store, so the
 * week loaded before it only has to be loaded again when a calendar actually changed.
 */
export async function runStartupSync({
  readSettings,
  syncContacts,
  reloadWeek,
  isCancelled,
}: StartupSync): Promise<void> {
  const before = await readSettings();
  if (isCancelled()) return;

  await syncContacts().catch(() => {});
  if (isCancelled()) return;

  const after = await readSettings();
  if (isCancelled()) return;

  if (calendarConfigChanged(before, after)) {
    reloadWeek();
  }
}

export function calendarConfigChanged(
  before: EmployeeSetting[] | null,
  after: EmployeeSetting[] | null,
): boolean {
  return calendarKey(before ?? []) !== calendarKey(after ?? []);
}

function calendarKey(settings: EmployeeSetting[]): string {
  return JSON.stringify(
    settings
      .filter((s) => s.zepPrimaryCalendar || s.zepAbsenceCalendar)
      .map((s) => [
        s.dayliteContactReference,
        s.zepPrimaryCalendar ?? "",
        s.zepAbsenceCalendar ?? "",
      ])
      .sort(([a], [b]) => a.localeCompare(b)),
  );
}
