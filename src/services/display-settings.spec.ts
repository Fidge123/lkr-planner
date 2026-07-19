import { beforeEach, describe, expect, it, mock } from "bun:test";
import { loadDisplaySettings, saveDisplaySettings } from "./display-settings";

interface DisplaySettings {
  hideNonPlannableEmployees?: boolean;
  showWeekend?: boolean;
}

let store: { displaySettings?: DisplaySettings } & Record<string, unknown>;

const mockLoadLocalStore = mock(() =>
  Promise.resolve({ status: "ok" as const, data: store }),
);
const mockSaveLocalStore = mock(
  (next: { displaySettings?: DisplaySettings } & Record<string, unknown>) => {
    store = next;
    return Promise.resolve({ status: "ok" as const, data: null });
  },
);

mock.module("../generated/tauri", () => ({
  commands: {
    loadLocalStore: mockLoadLocalStore,
    saveLocalStore: mockSaveLocalStore,
  },
}));

describe("display settings service", () => {
  beforeEach(() => {
    store = { apiEndpoints: {}, employeeSettings: [] };
    mockLoadLocalStore.mockClear();
    mockSaveLocalStore.mockClear();
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

  it("saving showWeekend preserves an already-stored hideNonPlannableEmployees value", async () => {
    store.displaySettings = {
      hideNonPlannableEmployees: false,
      showWeekend: false,
    };

    await saveDisplaySettings({ showWeekend: true });

    expect(store.displaySettings?.showWeekend).toBe(true);
    expect(store.displaySettings?.hideNonPlannableEmployees).toBe(false);
  });
});
