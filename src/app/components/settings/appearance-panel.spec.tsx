import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { AppearanceSettingsPanel } from "./appearance-panel";

describe("AppearanceSettingsPanel", () => {
  it("offers light, dark and system color schemes", () => {
    const html = renderToStaticMarkup(
      <AppearanceSettingsPanel onClose={() => {}} />,
    );

    expect(html).toContain("Darstellung");
    expect(html).toContain("Hell");
    expect(html).toContain("Dunkel");
    expect(html).toContain("System");
  });

  it("selects system by default", () => {
    const html = renderToStaticMarkup(
      <AppearanceSettingsPanel onClose={() => {}} />,
    );

    expect(html).toMatch(/<input[^>]*checked=""[^>]*value="system"/);
  });
});
