import {
  commands,
  type DisplaySettings,
  type LocalStore,
} from "../generated/tauri";
import { callCommand } from "./command-result";

export function displaySettingsOf(
  store: LocalStore | undefined,
): Required<DisplaySettings> {
  return {
    hideNonPlannableEmployees:
      store?.displaySettings?.hideNonPlannableEmployees ?? true,
    showWeekend: store?.displaySettings?.showWeekend ?? false,
  };
}

export function saveDisplaySettings(
  settings: Required<DisplaySettings>,
): Promise<null> {
  return callCommand(
    commands.saveDisplaySettings(settings),
    "Die Anzeige-Einstellung konnte nicht gespeichert werden.",
  );
}
