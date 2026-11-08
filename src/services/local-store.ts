import { commands, type LocalStore } from "../generated/tauri";
import { callCommand } from "./command-result";

export function loadLocalStore(): Promise<LocalStore> {
  return callCommand(
    commands.loadLocalStore(),
    "Die lokale Konfiguration konnte nicht geladen werden.",
  );
}
