import { describe, expect, it } from "bun:test";
import type { LocalStore } from "../generated/tauri";
import { displaySettingsOf } from "./display-settings";

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
