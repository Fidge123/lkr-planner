import { describe, expect, it } from "bun:test";
import { screen } from "@testing-library/react";
import { renderComponent } from "../../../test/render-component";
import { TelemetrySettingsPanel } from "./telemetry-panel";

function renderPanel() {
  return renderComponent(<TelemetrySettingsPanel onClose={() => {}} />)
    .container.textContent;
}

describe("TelemetrySettingsPanel", () => {
  it("renders the diagnostics section with a toggle", () => {
    renderPanel();

    expect(screen.getByText("Diagnose")).toBeDefined();
    expect(
      screen.getByRole("checkbox", { name: "Diagnosedaten senden" }),
    ).toBeDefined();
  });

  it("describes in German what is transmitted", () => {
    const text = renderPanel();

    expect(text).toContain("Fehler- und Leistungsdaten");
    expect(text).toContain("technische Details");
    expect(text).toContain("Projekt- oder Mitarbeiternamen");
    expect(text).toContain("IP-Adresse");
  });

  it("no longer calls the transmitted data anonymous", () => {
    expect(renderPanel()).not.toContain("anonym");
  });

  it("describes in German what is never transmitted", () => {
    expect(renderPanel()).toContain(
      "Passwörter, Zugangsschlüssel und Tokens werden nie übertragen.",
    );
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
