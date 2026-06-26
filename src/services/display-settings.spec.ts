import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  loadHideNonPlannableEmployees,
  loadShowWeekend,
  saveHideNonPlannableEmployees,
  saveShowWeekend,
} from "./display-settings";

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

  it("saveHideNonPlannableEmployees preserves an already-stored showWeekend value", async () => {
    store.displaySettings = {
      hideNonPlannableEmployees: true,
      showWeekend: true,
    };

    await saveHideNonPlannableEmployees(false);

    expect(store.displaySettings?.showWeekend).toBe(true);
    expect(store.displaySettings?.hideNonPlannableEmployees).toBe(false);
  });

  it("loadHideNonPlannableEmployees defaults to true when unset", async () => {
    expect(await loadHideNonPlannableEmployees()).toBe(true);
  });

  it("loadShowWeekend defaults to false when unset", async () => {
    expect(await loadShowWeekend()).toBe(false);
  });

  it("saveShowWeekend persists the value", async () => {
    await saveShowWeekend(true);
    expect(await loadShowWeekend()).toBe(true);
  });

  it("saveShowWeekend preserves an already-stored hideNonPlannableEmployees value", async () => {
    store.displaySettings = {
      hideNonPlannableEmployees: false,
      showWeekend: false,
    };

    await saveShowWeekend(true);

    expect(store.displaySettings?.showWeekend).toBe(true);
    expect(store.displaySettings?.hideNonPlannableEmployees).toBe(false);
  });
});
