import { useEffect, useRef, useState } from "react";
import {
  type CalendarCellEvent,
  commands,
  type DayliteProjectSummary,
} from "../../generated/tauri";
import { loadProjectsForAssignmentPicker } from "../../services/assignment-project-picker";

export function AssignmentModal({
  isOpen,
  assignment,
  employeeReference,
  date,
  onSave,
  onClose,
  showDeleteConfirm: initialShowDeleteConfirm = false,
  showUnsavedConfirm: initialShowUnsavedConfirm = false,
}: Props) {
  const isEditMode = assignment !== null;

  const [projects, setProjects] = useState<DayliteProjectSummary[]>([]);
  const [selectedProjectRef, setSelectedProjectRef] = useState<string>("");
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(
    initialShowDeleteConfirm,
  );
  const [showUnsavedConfirm, setShowUnsavedConfirm] = useState(
    initialShowUnsavedConfirm,
  );
  const [isDirty, setIsDirty] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    setErrorMessage(null);
    setIsSaving(false);
    setShowDeleteConfirm(initialShowDeleteConfirm);
    setShowUnsavedConfirm(initialShowUnsavedConfirm);
    setSelectedProjectRef("");
    setIsDirty(false);
    void loadProjectsForAssignmentPicker().then(setProjects);
  }, [isOpen, initialShowDeleteConfirm, initialShowUnsavedConfirm]);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;
    const handleCancel = (e: Event) => {
      e.preventDefault();
      requestClose();
    };
    dialog.addEventListener("cancel", handleCancel);
    return () => dialog.removeEventListener("cancel", handleCancel);
  });

  if (!isOpen) return null;

  const requestClose = () => {
    if (isSaving) return;
    if (isDirty) {
      setShowUnsavedConfirm(true);
      return;
    }
    onClose();
  };

  if (showUnsavedConfirm) {
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
              onClick={() => setShowUnsavedConfirm(false)}
            >
              Weiterbearbeiten
            </button>
            <button
              type="button"
              className="btn btn-sm btn-warning"
              onClick={onClose}
            >
              Verwerfen
            </button>
          </section>
        </section>
        <button
          type="button"
          className="modal-backdrop"
          onClick={() => setShowUnsavedConfirm(false)}
          aria-label="Dialog schließen"
        >
          Schließen
        </button>
      </dialog>
    );
  }

  if (showDeleteConfirm) {
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
              onClick={() => setShowDeleteConfirm(false)}
              disabled={isSaving}
            >
              Abbrechen
            </button>
            <button
              type="button"
              className="btn btn-sm btn-error"
              disabled={isSaving}
              onClick={async () => {
                if (!assignment?.href) return;
                setIsSaving(true);
                setErrorMessage(null);
                const result = await commands.deleteAssignment(assignment.href);
                if (result.status === "error") {
                  setErrorMessage(result.error);
                  setIsSaving(false);
                  return;
                }
                onSave();
              }}
            >
              {isSaving ? "Lösche..." : "Endgültig löschen"}
            </button>
          </section>
        </section>
        <button
          type="button"
          className="modal-backdrop"
          onClick={requestClose}
          aria-label="Dialog schließen"
        >
          Schließen
        </button>
      </dialog>
    );
  }

  const handleSave = async () => {
    setIsSaving(true);
    setErrorMessage(null);

    const project = projects.find((p) => p.self === selectedProjectRef);
    const projectName = project?.name ?? assignment?.title ?? "";

    let result: { status: string; error?: string };
    if (isEditMode && assignment.href) {
      result = await commands.updateAssignment(
        assignment.href,
        assignment.uid,
        date,
        selectedProjectRef || assignment.title,
        projectName,
      );
    } else {
      result = await commands.createAssignment(
        employeeReference,
        date,
        selectedProjectRef,
        projectName,
      );
    }

    if (result.status === "error") {
      setErrorMessage((result as { status: "error"; error: string }).error);
      setIsSaving(false);
      return;
    }
    onSave();
  };

  return (
    <dialog
      ref={dialogRef}
      className="modal modal-open"
      open
      aria-labelledby="assignment-modal-title"
    >
      <section className="modal-box max-w-md">
        <h2 id="assignment-modal-title" className="text-lg font-semibold">
          {isEditMode ? "Einsatz bearbeiten" : "Einsatz erstellen"}
        </h2>

        {errorMessage ? (
          <p className="mt-3 text-sm text-error">{errorMessage}</p>
        ) : null}

        <section className="mt-4 flex flex-col gap-3">
          {isEditMode ? (
            <p className="text-sm font-medium">{assignment.title}</p>
          ) : (
            <label className="form-control w-full">
              <span className="label-text mb-1">Projekt</span>
              <select
                className="select select-bordered w-full"
                value={selectedProjectRef}
                onChange={(e) => {
                  setSelectedProjectRef(e.target.value);
                  setIsDirty(true);
                }}
                disabled={isSaving}
              >
                <option value="">Projekt auswählen...</option>
                {projects.map((p) => (
                  <option key={p.self} value={p.self}>
                    {p.name}
                  </option>
                ))}
              </select>
            </label>
          )}
        </section>

        <section className="modal-action">
          {isEditMode ? (
            <button
              type="button"
              className="btn btn-sm btn-error mr-auto"
              onClick={() => setShowDeleteConfirm(true)}
              disabled={isSaving}
            >
              Löschen
            </button>
          ) : null}
          <button
            type="button"
            className="btn btn-sm"
            onClick={requestClose}
            disabled={isSaving}
          >
            Abbrechen
          </button>
          <button
            type="button"
            className="btn btn-sm btn-primary"
            onClick={handleSave}
            disabled={isSaving || (!isEditMode && !selectedProjectRef)}
          >
            {isSaving ? "Speichere..." : "Speichern"}
          </button>
        </section>
      </section>
      <button
        type="button"
        className="modal-backdrop"
        onClick={requestClose}
        aria-label="Dialog schließen"
      >
        Schließen
      </button>
    </dialog>
  );
}

interface Props {
  isOpen: boolean;
  assignment: CalendarCellEvent | null;
  employeeReference: string;
  date: string;
  onSave: () => void;
  onClose: () => void;
  showDeleteConfirm?: boolean;
  showUnsavedConfirm?: boolean;
}
