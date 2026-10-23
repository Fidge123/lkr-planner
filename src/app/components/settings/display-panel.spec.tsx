import { describe, expect, it } from "bun:test";
import { renderComponent } from "../../../test/render-component";
import { DisplaySettingsPanel } from "./display-panel";

describe("DisplaySettingsPanel", () => {
  it("renders the 'Wochenende anzeigen' toggle under Anzeige", () => {
    const html = renderComponent(<DisplaySettingsPanel onClose={() => {}} />)
      .container.innerHTML;

    expect(html).toContain("Anzeige");
    expect(html).toContain("Wochenende anzeigen");
  });
});
