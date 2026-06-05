import { commands } from "../generated/tauri";

// Mirrors the Rust default (DisplaySettings::default): hide non-plannable
// employees unless the user has explicitly turned the toggle off.
const DEFAULT_HIDE_NON_PLANNABLE_EMPLOYEES = true;

export async function loadHideNonPlannableEmployees(): Promise<boolean> {
  const result = await commands.loadLocalStore();
  if (result.status === "error") {
    throw new Error(result.error.userMessage);
  }

  return (
    result.data.displaySettings?.hideNonPlannableEmployees ??
    DEFAULT_HIDE_NON_PLANNABLE_EMPLOYEES
  );
}

export async function saveHideNonPlannableEmployees(
  hideNonPlannableEmployees: boolean,
): Promise<void> {
  const loaded = await commands.loadLocalStore();
  if (loaded.status === "error") {
    throw new Error(loaded.error.userMessage);
  }

  const saved = await commands.saveLocalStore({
    ...loaded.data,
    displaySettings: { hideNonPlannableEmployees },
  });
  if (saved.status === "error") {
    throw new Error(saved.error.userMessage);
  }
}
