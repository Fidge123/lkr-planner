import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { SettingsDialog } from "./settings-dialog";

describe("SettingsDialog", () => {
  it("lists a Diagnose section alongside the existing sections", () => {
    const html = renderToStaticMarkup(
      <SettingsDialog isOpen onClose={() => {}} />,
    );

    expect(html).toContain("Daylite");
    expect(html).toContain("ZEP");
    expect(html).toContain("Anzeige");
    expect(html).toContain("Diagnose");
  });

  it("renders nothing while closed", () => {
    const html = renderToStaticMarkup(
      <SettingsDialog isOpen={false} onClose={() => {}} />,
    );

    expect(html).toBe("");
  });
});
