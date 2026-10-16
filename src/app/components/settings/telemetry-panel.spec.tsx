import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { TelemetrySettingsPanel } from "./telemetry-panel";

describe("TelemetrySettingsPanel", () => {
  it("renders the diagnostics section with a toggle", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain("Diagnose");
    expect(html).toContain("Diagnosedaten senden");
    expect(html).toContain('type="checkbox"');
  });

  it("describes in German what is transmitted", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain("Fehler- und Leistungsdaten");
    expect(html).toContain("technische Details");
    expect(html).toContain("Projekt- oder Mitarbeiternamen");
    expect(html).toContain("IP-Adresse");
  });

  it("no longer calls the transmitted data anonymous", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).not.toContain("anonym");
  });

  it("describes in German what is never transmitted", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain(
      "Passwörter, Zugangsschlüssel und Tokens werden nie übertragen.",
    );
  });

  it("shows telemetry as disabled before the stored state is loaded", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).not.toContain("checked=");
  });
});
