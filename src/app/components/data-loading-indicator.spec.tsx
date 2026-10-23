import { describe, expect, it } from "bun:test";
import { screen } from "@testing-library/react";
import { renderComponent } from "../../test/render-component";
import { DataLoadingIndicator } from "./data-loading-indicator";

describe("DataLoadingIndicator", () => {
  it("shows a spinner with the german loading text while loading", () => {
    const { container } = renderComponent(<DataLoadingIndicator isLoading />);

    expect(container.querySelector(".loading-spinner")).not.toBeNull();
    expect(screen.getByText("Daten werden geladen...")).toBeDefined();
  });

  it("renders nothing when nothing is loading", () => {
    const { container } = renderComponent(
      <DataLoadingIndicator isLoading={false} />,
    );

    expect(container.innerHTML).toBe("");
  });
});
