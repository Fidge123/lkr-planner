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

    expect(html).toContain("anonyme Fehler- und Leistungsdaten");
  });

  it("describes in German what is never transmitted", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain("Keine Projekt-, Kontakt- oder Zugangsdaten");
  });

  it("shows telemetry as disabled before the stored state is loaded", () => {
    const html = renderToStaticMarkup(
      <TelemetrySettingsPanel onClose={() => {}} />,
    );

    expect(html).not.toContain("checked=");
  });
});
