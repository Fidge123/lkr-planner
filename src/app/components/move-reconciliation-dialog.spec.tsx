import { beforeEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { commandFailed, commandOk } from "../../test/app-error";
import { mockCommands } from "../../test/mock-commands";
import { renderComponent } from "../../test/render-component";
import {
  type MoveReconciliation,
  MoveReconciliationDialog,
} from "./move-reconciliation-dialog";

const mockDeleteAssignment = mock((_href: string, _override: boolean) =>
  commandOk(null),
);
const onResolved = mock(() => {});

mockCommands({ deleteAssignment: mockDeleteAssignment });

const reconciliation: MoveReconciliation = {
  newHref: "/calendars/target/uid-1.ics",
  sourceHref: "/calendars/source/uid-1.ics",
};

function renderDialog(shown: MoveReconciliation | null = reconciliation) {
  return renderComponent(
    <MoveReconciliationDialog reconciliation={shown} onResolved={onResolved} />,
  ).container.innerHTML;
}

describe("MoveReconciliationDialog", () => {
  beforeEach(() => {
    mockDeleteAssignment.mockClear();
    onResolved.mockClear();
  });

  it("renders nothing when reconciliation is null", () => {
    expect(renderDialog(null)).toBe("");
  });

  it("renders no modal-backdrop, so the dialog blocks until a choice is made", () => {
    expect(renderDialog()).not.toContain("modal-backdrop");
    expect(screen.getByText("Einsatz doppelt vorhanden")).toBeDefined();
  });

  it("deletes the original again, even if it is protected", async () => {
    renderDialog();

    fireEvent.click(screen.getByText("Original erneut löschen"));

    await waitFor(() => expect(onResolved).toHaveBeenCalled());
    expect(mockDeleteAssignment).toHaveBeenCalledWith(
      reconciliation.sourceHref,
      true,
    );
  });

  it("keeps the original and deletes the copy", async () => {
    renderDialog();

    fireEvent.click(screen.getByText("Original behalten, Kopie löschen"));

    await waitFor(() => expect(onResolved).toHaveBeenCalled());
    expect(mockDeleteAssignment).toHaveBeenCalledWith(
      reconciliation.newHref,
      true,
    );
  });

  it("keeps both without deleting", () => {
    renderDialog();

    fireEvent.click(screen.getByText("Beide behalten"));

    expect(onResolved).toHaveBeenCalled();
    expect(mockDeleteAssignment).not.toHaveBeenCalled();
  });

  it("stays open with the German error and enabled choices when deleting fails", async () => {
    mockDeleteAssignment.mockImplementationOnce(() =>
      commandFailed("Der Einsatz konnte nicht gelöscht werden."),
    );
    renderDialog();

    fireEvent.click(screen.getByText("Original erneut löschen"));

    expect(
      await screen.findByText("Der Einsatz konnte nicht gelöscht werden."),
    ).toBeDefined();
    expect(onResolved).not.toHaveBeenCalled();
    for (const button of screen.getAllByRole<HTMLButtonElement>("button")) {
      expect(button.disabled).toBe(false);
    }
  });
});
