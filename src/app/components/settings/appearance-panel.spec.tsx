import { describe, expect, it } from "bun:test";
import { screen } from "@testing-library/react";
import { renderComponent } from "../../../test/render-component";
import { AppearanceSettingsPanel } from "./appearance-panel";

describe("AppearanceSettingsPanel", () => {
  it("offers light, dark and system color schemes", () => {
    renderComponent(<AppearanceSettingsPanel onClose={() => {}} />);

    expect(screen.getByText("Darstellung")).toBeDefined();
    expect(screen.getByRole("radio", { name: "Hell" })).toBeDefined();
    expect(screen.getByRole("radio", { name: "Dunkel" })).toBeDefined();
    expect(screen.getByRole("radio", { name: "System" })).toBeDefined();
  });

  it("selects system by default", () => {
    renderComponent(<AppearanceSettingsPanel onClose={() => {}} />);

    expect(
      screen.getByRole<HTMLInputElement>("radio", { name: "System" }).checked,
    ).toBe(true);
  });
});
