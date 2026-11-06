import { describe, expect, it, mock } from "bun:test";
import { fireEvent } from "@testing-library/react";
import { renderComponent } from "../../../test/render-component";
import { SettingsDialog } from "./settings-dialog";

describe("SettingsDialog", () => {
  it("renders nothing while closed", () => {
    const { container } = renderComponent(
      <SettingsDialog isOpen={false} onClose={() => {}} />,
    );

    expect(container.innerHTML).toBe("");
  });

  it("closes on Escape only while open", () => {
    const onClose = mock(() => {});
    const { rerender } = renderComponent(
      <SettingsDialog isOpen={false} onClose={onClose} />,
    );

    fireEvent.keyDown(document.body, { key: "Escape" });
    expect(onClose).not.toHaveBeenCalled();

    rerender(<SettingsDialog isOpen onClose={onClose} />);
    fireEvent.keyDown(document.body, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
