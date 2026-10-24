import { describe, expect, it, mock } from "bun:test";
import type { LocalStore } from "../generated/tauri";
import { mockCommands } from "../test/mock-commands";
import { displaySettingsOf, saveDisplaySettings } from "./display-settings";

const mockSaveDisplaySettings = mock(() =>
  Promise.resolve({ status: "ok" as const, data: null }),
);

mockCommands({ saveDisplaySettings: mockSaveDisplaySettings });

const store = (displaySettings?: LocalStore["displaySettings"]) =>
  ({ displaySettings }) as LocalStore;

describe("displaySettingsOf", () => {
  it("uses the defaults for a store saved before the settings existed", () => {
    expect(displaySettingsOf(store())).toEqual({
      hideNonPlannableEmployees: true,
      showWeekend: false,
    });
  });

  it("keeps the stored values", () => {
    expect(
      displaySettingsOf(
        store({ hideNonPlannableEmployees: false, showWeekend: true }),
      ),
    ).toEqual({ hideNonPlannableEmployees: false, showWeekend: true });
  });
});

describe("saveDisplaySettings", () => {
  it("hands both settings to the backend", async () => {
    await saveDisplaySettings({
      hideNonPlannableEmployees: false,
      showWeekend: true,
    });

    expect(mockSaveDisplaySettings).toHaveBeenCalledWith({
      hideNonPlannableEmployees: false,
      showWeekend: true,
    });
  });
});
