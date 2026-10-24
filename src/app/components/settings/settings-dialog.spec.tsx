import { describe, expect, it } from "bun:test";
import { screen } from "@testing-library/react";
import { renderComponent } from "../../../test/render-component";
import { SettingsDialog } from "./settings-dialog";

describe("SettingsDialog", () => {
  it("lists a Diagnose section alongside the existing sections", () => {
    renderComponent(<SettingsDialog isOpen onClose={() => {}} />);

    for (const section of [
      "Daylite",
      "ZEP",
      "Anzeige",
      "Darstellung",
      "Diagnose",
    ]) {
      expect(screen.getByRole("button", { name: section })).toBeDefined();
    }
  });

  it("renders nothing while closed", () => {
    const { container } = renderComponent(
      <SettingsDialog isOpen={false} onClose={() => {}} />,
    );

    expect(container.innerHTML).toBe("");
  });
});
