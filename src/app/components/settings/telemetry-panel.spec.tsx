import { beforeEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { commandFailed, commandOk } from "../../../test/app-error";
import { mockCommands } from "../../../test/mock-commands";
import { renderComponent } from "../../../test/render-component";
import { telemetrySettingsQuery } from "../../hooks/use-telemetry-settings";
import { TelemetrySettingsPanel } from "./telemetry-panel";

let storedEnabled = false;

const mockSetEnabled = mock((enabled: boolean) => {
  storedEnabled = enabled;
  return commandOk({ enabled });
});

mockCommands({
  telemetryGetSettings: () => Promise.resolve({ enabled: storedEnabled }),
  telemetrySetEnabled: mockSetEnabled,
});

const toggle = () =>
  screen.getByRole<HTMLInputElement>("checkbox", {
    name: "Diagnosedaten senden",
  });

describe("TelemetrySettingsPanel", () => {
  beforeEach(() => {
    storedEnabled = false;
  });

  it("renders the diagnostics section with a toggle", () => {
    renderComponent(<TelemetrySettingsPanel onClose={() => {}} />);

    expect(screen.getByText("Diagnose")).toBeDefined();
    expect(toggle()).toBeDefined();
  });

  it("shows telemetry as disabled before the stored state is loaded", () => {
    storedEnabled = true;

    renderComponent(<TelemetrySettingsPanel onClose={() => {}} />);

    expect(toggle().checked).toBe(false);
  });

  it("shows the stored setting again with a German message when saving fails", async () => {
    mockSetEnabled.mockImplementationOnce(() =>
      commandFailed("Die Einstellung konnte nicht gespeichert werden."),
    );
    renderComponent(<TelemetrySettingsPanel onClose={() => {}} />, [
      [telemetrySettingsQuery.queryKey, false],
    ]);

    fireEvent.click(toggle());

    expect(
      await screen.findByText(
        "Die Einstellung konnte nicht gespeichert werden.",
      ),
    ).toBeDefined();
    await waitFor(() => expect(toggle().checked).toBe(false));
  });
});
