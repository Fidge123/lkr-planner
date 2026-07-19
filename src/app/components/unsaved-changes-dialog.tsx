export function UnsavedChangesDialog({ onContinueEditing, onDiscard }: Props) {
  return (
    <dialog
      className="modal modal-open"
      open
      aria-labelledby="assignment-unsaved-title"
    >
      <section className="modal-box max-w-sm">
        <h2 id="assignment-unsaved-title" className="text-lg font-semibold">
          Ungespeicherte Änderungen
        </h2>
        <p className="mt-3 text-sm">
          Es gibt ungespeicherte Änderungen. Möchten Sie diese verwerfen?
        </p>
        <section className="modal-action">
          <button
            type="button"
            className="btn btn-sm"
            onClick={onContinueEditing}
          >
            Weiterbearbeiten
          </button>
          <button
            type="button"
            className="btn btn-sm btn-warning"
            onClick={onDiscard}
          >
            Verwerfen
          </button>
        </section>
      </section>
      <button
        type="button"
        className="modal-backdrop"
        onClick={onContinueEditing}
        aria-label="Dialog schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

interface Props {
  onContinueEditing: () => void;
  onDiscard: () => void;
}
