import { beforeEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, screen } from "@testing-library/react";
import type {
  DayliteRefreshTokenRequest,
  LocalStore,
} from "../../../generated/tauri";
import { commandOk } from "../../../test/app-error";
import { mockCommands } from "../../../test/mock-commands";
import { renderComponent } from "../../../test/render-component";
import { localStoreQuery } from "../../hooks/use-local-store";
import { DayliteSettingsPanel } from "./daylite-panel";

const initialStore: LocalStore = {
  apiEndpoints: {
    dayliteBaseUrl: "https://daylite.example/v1/",
    planradarBaseUrl: "",
  },
  employeeSettings: [],
  displaySettings: { hideNonPlannableEmployees: false, showWeekend: false },
  dayliteCache: { lastSyncedAt: null, contacts: [] },
};
let storedStore = initialStore;

mockCommands({
  loadLocalStore: () => commandOk(storedStore),
  dayliteConnectRefreshToken: mock(
    ({ baseUrl }: DayliteRefreshTokenRequest) => {
      storedStore = {
        ...storedStore,
        apiEndpoints: { ...storedStore.apiEndpoints, dayliteBaseUrl: baseUrl },
      };
      return commandOk({ hasAccessToken: true, hasRefreshToken: true });
    },
  ),
});

function renderPanel() {
  return renderComponent(<DayliteSettingsPanel onClose={() => {}} />, [
    [localStoreQuery.queryKey, storedStore],
  ]);
}

describe("DayliteSettingsPanel", () => {
  beforeEach(() => {
    storedStore = initialStore;
  });

  it("fills in the stored Daylite API URL", () => {
    renderPanel();

    expect(
      screen.getByDisplayValue("https://daylite.example/v1"),
    ).toBeDefined();
  });

  it("shows the saved URL when the panel is opened again", async () => {
    const { rerender } = renderPanel();
    fireEvent.change(screen.getByDisplayValue("https://daylite.example/v1"), {
      target: { value: "https://daylite.example/v2" },
    });
    fireEvent.change(screen.getByPlaceholderText("Refresh-Token eingeben"), {
      target: { value: "refresh-token-123" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Speichern" }));
    await screen.findByText("Daylite-Konfiguration wurde aktualisiert.");

    rerender(<DayliteSettingsPanel key="reopened" onClose={() => {}} />);

    expect(
      await screen.findByDisplayValue("https://daylite.example/v2"),
    ).toBeDefined();
  });
});
