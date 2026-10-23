import { beforeEach, describe, expect, it, mock } from "bun:test";
import { mockCommands } from "../test/mock-commands";
import { loadDisplaySettings, saveDisplaySettings } from "./display-settings";

interface DisplaySettings {
  hideNonPlannableEmployees?: boolean;
  showWeekend?: boolean;
}

let store: { displaySettings?: DisplaySettings } & Record<string, unknown>;

const mockLoadLocalStore = mock(() =>
  Promise.resolve({ status: "ok" as const, data: store }),
);
const mockSaveDisplaySettings = mock((settings: DisplaySettings) => {
  store = { ...store, displaySettings: settings };
  return Promise.resolve({ status: "ok" as const, data: null });
});

mockCommands({
  loadLocalStore: mockLoadLocalStore,
  saveDisplaySettings: mockSaveDisplaySettings,
});

describe("display settings service", () => {
  beforeEach(() => {
    store = { apiEndpoints: {}, employeeSettings: [] };
    mockLoadLocalStore.mockClear();
    mockSaveDisplaySettings.mockClear();
  });

  it("saving one setting preserves the other stored settings", async () => {
    store.displaySettings = {
      hideNonPlannableEmployees: true,
      showWeekend: true,
    };

    await saveDisplaySettings({ hideNonPlannableEmployees: false });

    expect(store.displaySettings?.showWeekend).toBe(true);
    expect(store.displaySettings?.hideNonPlannableEmployees).toBe(false);
  });

  it("loads the backend defaults when nothing is stored", async () => {
    const settings = await loadDisplaySettings();

    expect(settings.hideNonPlannableEmployees).toBe(true);
    expect(settings.showWeekend).toBe(false);
  });

  it("persists a saved value", async () => {
    await saveDisplaySettings({ showWeekend: true });

    expect((await loadDisplaySettings()).showWeekend).toBe(true);
  });
});
