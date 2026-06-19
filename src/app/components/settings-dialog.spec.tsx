import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { DisplaySettingsPanel } from "./settings-dialog";

describe("DisplaySettingsPanel", () => {
  it("renders the 'Wochenende anzeigen' toggle under Anzeige", () => {
    const html = renderToStaticMarkup(
      <DisplaySettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain("Anzeige");
    expect(html).toContain("Wochenende anzeigen");
  });
});
