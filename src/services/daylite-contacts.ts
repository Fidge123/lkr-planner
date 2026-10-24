import { commands, type PlanningContactRecord } from "../generated/tauri";
import { callCommand } from "./command-result";

export function loadDayliteContacts(): Promise<PlanningContactRecord[]> {
  return callCommand(
    commands.dayliteListContacts(),
    "Die Mitarbeiter konnten nicht von Daylite geladen werden.",
  );
}

/** The contacts of the last successful load, kept by the backend. */
export function loadCachedDayliteContacts(): Promise<PlanningContactRecord[]> {
  return callCommand(
    commands.dayliteListCachedContacts(),
    "Die gespeicherten Mitarbeiter konnten nicht geladen werden.",
  );
}
