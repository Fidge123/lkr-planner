import { describe, expect, it } from "bun:test";
import { renderComponent } from "../../../test/render-component";
import { SettingsDialog } from "./settings-dialog";

describe("SettingsDialog", () => {
  it("renders nothing while closed", () => {
    const { container } = renderComponent(
      <SettingsDialog isOpen={false} onClose={() => {}} />,
    );

    expect(container.innerHTML).toBe("");
  });
});
