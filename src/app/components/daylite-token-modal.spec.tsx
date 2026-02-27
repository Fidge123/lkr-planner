import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { DayliteTokenModal } from "./daylite-token-modal";

describe("daylite token modal", () => {
  it("renders nothing when closed", () => {
    const html = renderToStaticMarkup(
      <DayliteTokenModal isOpen={false} onClose={() => {}} />,
    );

    expect(html).toBe("");
  });

  it("renders daylite configuration form when open", () => {
    const html = renderToStaticMarkup(
      <DayliteTokenModal isOpen onClose={() => {}} />,
    );

    expect(html).toContain("<dialog");
    expect(html).toContain("Daylite-Konfiguration");
    expect(html).toContain("Refresh-Token");
    expect(html).toContain("Token abrufen");
  });
});
