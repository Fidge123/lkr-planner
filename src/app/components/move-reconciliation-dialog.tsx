import { useState } from "react";
import { commands } from "../../generated/tauri";

export function MoveReconciliationDialog({
  reconciliation,
  onResolved,
}: Props) {
  const [isSaving, setIsSaving] = useState(false);
  const [pendingChoice, setPendingChoice] =
    useState<ReconciliationChoice | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  if (!reconciliation) return null;

  const resolve = async (choice: ReconciliationChoice) => {
    if (choice === "keepBoth") {
      onResolved();
      return;
    }

    setIsSaving(true);
    setPendingChoice(choice);
    setErrorMessage(null);
    try {
      const result = await commands.deleteAssignment(
        hrefToDelete(choice, reconciliation),
      );
      if (result.status === "error") {
        setErrorMessage(result.error);
        setIsSaving(false);
        setPendingChoice(null);
        return;
      }
      onResolved();
    } catch {
      // The generated bindings re-throw Error-typed rejections (IPC failures);
      // without this the dialog would stay disabled forever.
      setErrorMessage("Der Einsatz konnte nicht gelöscht werden.");
      setIsSaving(false);
      setPendingChoice(null);
    }
  };

  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="move-reconciliation-title"
    >
      <section className="modal-box max-w-sm">
        <h2 id="move-reconciliation-title" className="text-lg font-semibold">
          Einsatz doppelt vorhanden
        </h2>
        <p className="mt-3 text-sm">
          Der Einsatz wurde in den Zielkalender kopiert, aber das Original
          konnte nicht gelöscht werden. Der Einsatz existiert jetzt doppelt.
        </p>
        {errorMessage ? (
          <p className="mt-3 text-sm text-error">{errorMessage}</p>
        ) : null}
        <section className="modal-action flex-col items-stretch">
          <button
            type="button"
            className="btn btn-sm btn-primary"
            disabled={isSaving}
            onClick={() => resolve("retryDeleteSource")}
          >
            {isSaving && pendingChoice === "retryDeleteSource"
              ? "Lösche..."
              : "Original erneut löschen"}
          </button>
          <button
            type="button"
            className="btn btn-sm"
            disabled={isSaving}
            onClick={() => resolve("keepOldDeleteNew")}
          >
            {isSaving && pendingChoice === "keepOldDeleteNew"
              ? "Lösche..."
              : "Original behalten, Kopie löschen"}
          </button>
          <button
            type="button"
            className="btn btn-sm btn-ghost"
            disabled={isSaving}
            onClick={() => resolve("keepBoth")}
          >
            Beide behalten
          </button>
        </section>
      </section>
    </dialog>
  );
}

interface Props {
  reconciliation: MoveReconciliation | null;
  onResolved: () => void;
}

export interface MoveReconciliation {
  newHref: string;
  sourceHref: string;
}

export type ReconciliationChoice =
  | "retryDeleteSource"
  | "keepOldDeleteNew"
  | "keepBoth";

// Maps a deleting choice to the href that must be deleted, so the component
// and its tests share a single source of truth for this mapping.
export function hrefToDelete(
  choice: Exclude<ReconciliationChoice, "keepBoth">,
  reconciliation: MoveReconciliation,
): string {
  return choice === "retryDeleteSource"
    ? reconciliation.sourceHref
    : reconciliation.newHref;
}
