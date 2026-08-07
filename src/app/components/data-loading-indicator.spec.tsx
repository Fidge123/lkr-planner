import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { DataLoadingIndicator } from "./data-loading-indicator";

describe("DataLoadingIndicator", () => {
  it("shows a spinner with the german loading text while loading", () => {
    const html = renderToStaticMarkup(<DataLoadingIndicator isLoading />);

    expect(html).toContain("loading loading-spinner");
    expect(html).toContain("Daten werden geladen...");
  });

  it("renders nothing when nothing is loading", () => {
    expect(
      renderToStaticMarkup(<DataLoadingIndicator isLoading={false} />),
    ).toBe("");
  });
});
