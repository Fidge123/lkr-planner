import { describe, expect, it } from "bun:test";
import { screen } from "@testing-library/react";
import { renderComponent } from "../../../test/render-component";
import { TelemetrySettingsPanel } from "./telemetry-panel";

function renderPanel() {
  renderComponent(<TelemetrySettingsPanel onClose={() => {}} />);
}

describe("TelemetrySettingsPanel", () => {
  it("renders the diagnostics section with a toggle", () => {
    renderPanel();

    expect(screen.getByText("Diagnose")).toBeDefined();
    expect(
      screen.getByRole("checkbox", { name: "Diagnosedaten senden" }),
    ).toBeDefined();
  });

  it("shows telemetry as disabled before the stored state is loaded", () => {
    renderPanel();

    expect(
      screen.getByRole<HTMLInputElement>("checkbox", {
        name: "Diagnosedaten senden",
      }).checked,
    ).toBe(false);
  });
});
