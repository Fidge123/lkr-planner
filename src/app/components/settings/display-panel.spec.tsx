import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { DisplaySettingsPanel } from "./display-panel";

describe("DisplaySettingsPanel", () => {
  it("renders the 'Wochenende anzeigen' toggle under Anzeige", () => {
    const html = renderToStaticMarkup(
      <DisplaySettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain("Anzeige");
    expect(html).toContain("Wochenende anzeigen");
  });

  it("renders the weekend toggle without a description (avoids modal overflow)", () => {
    const html = renderToStaticMarkup(
      <DisplaySettingsPanel onClose={() => {}} />,
    );

    expect(html).not.toContain("Samstag und Sonntag");
  });
});
