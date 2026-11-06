import { beforeEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import type { DisplaySettings, LocalStore } from "../../../generated/tauri";
import { commandFailed, commandOk } from "../../../test/app-error";
import { mockCommands } from "../../../test/mock-commands";
import { renderComponent } from "../../../test/render-component";
import { localStoreQuery } from "../../hooks/use-local-store";
import { DisplaySettingsPanel } from "./display-panel";

const initialStore: LocalStore = {
  apiEndpoints: { dayliteBaseUrl: "", planradarBaseUrl: "" },
  employeeSettings: [],
  displaySettings: { hideNonPlannableEmployees: true, showWeekend: false },
  dayliteCache: { lastSyncedAt: null, contacts: [] },
};
let storedStore = initialStore;

const mockSaveDisplaySettings = mock((displaySettings: DisplaySettings) => {
  storedStore = { ...storedStore, displaySettings };
  return commandOk(null);
});

mockCommands({
  saveDisplaySettings: mockSaveDisplaySettings,
  loadLocalStore: () => commandOk(storedStore),
});

function renderPanel() {
  renderComponent(<DisplaySettingsPanel onClose={() => {}} />, [
    [localStoreQuery.queryKey, storedStore],
  ]);
  return screen.getByRole<HTMLInputElement>("checkbox", {
    name: "Wochenende anzeigen",
  });
}

describe("DisplaySettingsPanel", () => {
  beforeEach(() => {
    storedStore = initialStore;
    mockSaveDisplaySettings.mockClear();
  });

  it("renders the 'Wochenende anzeigen' toggle under Anzeige", () => {
    expect(renderPanel().checked).toBe(false);
    expect(screen.getByText("Anzeige")).toBeDefined();
  });

  it("saves a toggled setting together with the other one", async () => {
    const toggle = renderPanel();

    fireEvent.click(toggle);

    expect(toggle.checked).toBe(true);
    expect(mockSaveDisplaySettings).toHaveBeenCalledWith({
      hideNonPlannableEmployees: true,
      showWeekend: true,
    });
    await waitFor(() => expect(toggle.disabled).toBe(false));
    expect(toggle.checked).toBe(true);
  });

  it("shows the stored setting again with a German message when saving fails", async () => {
    mockSaveDisplaySettings.mockImplementationOnce(() =>
      commandFailed("Die Einstellungen konnten nicht gespeichert werden."),
    );
    const toggle = renderPanel();

    fireEvent.click(toggle);

    expect(
      await screen.findByText(
        "Die Einstellungen konnten nicht gespeichert werden.",
      ),
    ).toBeDefined();
    await waitFor(() => expect(toggle.checked).toBe(false));
  });
});
