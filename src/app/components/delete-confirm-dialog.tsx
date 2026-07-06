export function DeleteConfirmDialog({
  isDeleting,
  errorMessage,
  onCancel,
  onConfirm,
  onRequestClose,
}: Props) {
  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="assignment-delete-title"
    >
      <section className="modal-box max-w-sm">
        <h2 id="assignment-delete-title" className="text-lg font-semibold">
          Einsatz löschen
        </h2>
        <p className="mt-3 text-sm">
          Soll dieser Einsatz wirklich gelöscht werden?
        </p>
        {errorMessage ? (
          <p className="mt-3 text-sm text-error">{errorMessage}</p>
        ) : null}
        <section className="modal-action">
          <button
            type="button"
            className="btn btn-sm"
            onClick={onCancel}
            disabled={isDeleting}
          >
            Abbrechen
          </button>
          <button
            type="button"
            className="btn btn-sm btn-error"
            disabled={isDeleting}
            onClick={onConfirm}
          >
            {isDeleting ? "Lösche..." : "Endgültig löschen"}
          </button>
        </section>
      </section>
      <button
        type="button"
        className="modal-backdrop"
        onClick={onRequestClose}
        aria-label="Dialog schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

interface Props {
  isDeleting: boolean;
  errorMessage: string | null;
  onCancel: () => void;
  onConfirm: () => void;
  onRequestClose: () => void;
}
