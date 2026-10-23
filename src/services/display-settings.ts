import { commands, type DisplaySettings } from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";
import { loadLocalStore } from "./local-store";

const defaultDisplaySettings: Required<DisplaySettings> = {
  hideNonPlannableEmployees: true,
  showWeekend: false,
};

export async function loadDisplaySettings(): Promise<
  Required<DisplaySettings>
> {
  const store = await loadLocalStore();
  return {
    hideNonPlannableEmployees:
      store.displaySettings?.hideNonPlannableEmployees ??
      defaultDisplaySettings.hideNonPlannableEmployees,
    showWeekend:
      store.displaySettings?.showWeekend ?? defaultDisplaySettings.showWeekend,
  };
}

export async function saveDisplaySettings(
  patch: Partial<DisplaySettings>,
): Promise<void> {
  const store = await loadLocalStore();
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
