import { commands, type DisplaySettings } from "../generated/tauri";

// Mirrors the Rust default (DisplaySettings::default): hide non-plannable
// employees unless the user has explicitly turned the toggle off.
const DEFAULT_HIDE_NON_PLANNABLE_EMPLOYEES = true;

// Mirrors the Rust default (DisplaySettings::default): the weekend is hidden
// unless the user has explicitly turned the toggle on.
const DEFAULT_SHOW_WEEKEND = false;

const DEFAULT_DISPLAY_SETTINGS: DisplaySettings = {
  hideNonPlannableEmployees: DEFAULT_HIDE_NON_PLANNABLE_EMPLOYEES,
  showWeekend: DEFAULT_SHOW_WEEKEND,
};

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
    // Merge into the existing display settings so saving one field does not drop
    // the others (e.g. showWeekend).
    displaySettings: {
      ...(loaded.data.displaySettings ?? DEFAULT_DISPLAY_SETTINGS),
      hideNonPlannableEmployees,
    },
  });
  if (saved.status === "error") {
    throw new Error(saved.error.userMessage);
  }
}

export async function loadShowWeekend(): Promise<boolean> {
  const result = await commands.loadLocalStore();
  if (result.status === "error") {
    throw new Error(result.error.userMessage);
  }

  return result.data.displaySettings?.showWeekend ?? DEFAULT_SHOW_WEEKEND;
}

export async function saveShowWeekend(showWeekend: boolean): Promise<void> {
  const loaded = await commands.loadLocalStore();
  if (loaded.status === "error") {
    throw new Error(loaded.error.userMessage);
  }

  const saved = await commands.saveLocalStore({
    ...loaded.data,
    // Merge into the existing display settings so saving one field does not drop
    // the others (e.g. hideNonPlannableEmployees).
    displaySettings: {
      ...(loaded.data.displaySettings ?? DEFAULT_DISPLAY_SETTINGS),
      showWeekend,
    },
  });
  if (saved.status === "error") {
    throw new Error(saved.error.userMessage);
  }
}
