import { commands, type DisplaySettings } from "../generated/tauri";
import { unwrapCommandResult } from "./daylite-service-helpers";

// Mirrors DisplaySettings::default() in the Rust backend.
const defaultDisplaySettings: DisplaySettings = {
  hideNonPlannableEmployees: true,
  showWeekend: false,
};

const loadErrorMessage =
  "Die lokale Konfiguration konnte nicht geladen werden.";

export async function loadDisplaySettings(): Promise<DisplaySettings> {
  const store = unwrapCommandResult(
    await commands.loadLocalStore(),
    loadErrorMessage,
  );
  return { ...defaultDisplaySettings, ...store.displaySettings };
}

export async function saveDisplaySettings(
  patch: Partial<DisplaySettings>,
): Promise<void> {
  const store = unwrapCommandResult(
    await commands.loadLocalStore(),
    loadErrorMessage,
  );
  unwrapCommandResult(
    await commands.saveLocalStore({
      ...store,
      displaySettings: {
        ...defaultDisplaySettings,
        ...store.displaySettings,
        ...patch,
      },
    }),
    "Die Anzeige-Einstellung konnte nicht gespeichert werden.",
  );
}
