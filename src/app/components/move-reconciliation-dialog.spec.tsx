import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import {
  hrefToDelete,
  type MoveReconciliation,
  MoveReconciliationDialog,
} from "./move-reconciliation-dialog";

describe("MoveReconciliationDialog", () => {
  const reconciliation: MoveReconciliation = {
    newHref: "/calendars/target/uid-1.ics",
    sourceHref: "/calendars/source/uid-1.ics",
  };

  it("renders the title and all three option labels", () => {
    const html = renderToStaticMarkup(
      <MoveReconciliationDialog
        reconciliation={reconciliation}
        onResolved={() => {}}
      />,
    );

    expect(html).toContain("Einsatz doppelt vorhanden");
    expect(html).toContain("Original erneut löschen");
    expect(html).toContain("Original behalten, Kopie löschen");
    expect(html).toContain("Beide behalten");
  });

  it("renders nothing when reconciliation is null", () => {
    const html = renderToStaticMarkup(
      <MoveReconciliationDialog reconciliation={null} onResolved={() => {}} />,
    );

    expect(html).toBe("");
  });

  it("renders no modal-backdrop, so the dialog blocks until a choice is made", () => {
    const html = renderToStaticMarkup(
      <MoveReconciliationDialog
        reconciliation={reconciliation}
        onResolved={() => {}}
      />,
    );

    expect(html).not.toContain("modal-backdrop");
  });
});

describe("hrefToDelete", () => {
  const reconciliation: MoveReconciliation = {
    newHref: "/calendars/target/uid-1.ics",
    sourceHref: "/calendars/source/uid-1.ics",
  };

  it("returns the source href for retryDeleteSource", () => {
    expect(hrefToDelete("retryDeleteSource", reconciliation)).toBe(
      reconciliation.sourceHref,
    );
  });

  it("returns the new href for keepOldDeleteNew", () => {
    expect(hrefToDelete("keepOldDeleteNew", reconciliation)).toBe(
      reconciliation.newHref,
    );
  });
});
