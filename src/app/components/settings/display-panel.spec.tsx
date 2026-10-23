import { describe, expect, it } from "bun:test";
import { renderWithQueries } from "../../../test/render-with-queries";
import { DisplaySettingsPanel } from "./display-panel";

describe("DisplaySettingsPanel", () => {
  it("renders the 'Wochenende anzeigen' toggle under Anzeige", () => {
    const html = renderWithQueries(<DisplaySettingsPanel onClose={() => {}} />);

    expect(html).toContain("Anzeige");
    expect(html).toContain("Wochenende anzeigen");
  });
});
