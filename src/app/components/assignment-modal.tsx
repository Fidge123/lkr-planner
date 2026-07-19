import type { CalendarCellEvent } from "../../generated/tauri";
import { useAssignmentModal } from "../hooks/use-assignment-modal";
import type { ModalSaveAction } from "../next-day-quick-add";
import { DeleteConfirmDialog } from "./delete-confirm-dialog";
import { ProjectResultList, SuggestionEmptyState } from "./project-result-list";
import { UnsavedChangesDialog } from "./unsaved-changes-dialog";

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
  const modal = useAssignmentModal({
    isOpen,
    assignment,
    employeeReference,
    date,
    onSave,
    onClose,
    initialShowDeleteConfirm,
    initialShowUnsavedConfirm,
  });

  if (!isOpen) return null;

  if (modal.showUnsavedConfirm) {
    return (
      <UnsavedChangesDialog
        onContinueEditing={modal.continueEditing}
        onDiscard={onClose}
      />
    );
  }

  if (modal.showDeleteConfirm) {
    return (
      <DeleteConfirmDialog
        isDeleting={modal.isSaving}
        errorMessage={modal.errorMessage}
        onCancel={modal.cancelDeleteConfirm}
        onConfirm={modal.handleDelete}
        onRequestClose={modal.requestClose}
      />
    );
  }

  return (
    <dialog
      ref={modal.dialogRef}
      className="modal modal-open"
      open
      aria-labelledby="assignment-modal-title"
    >
      <section className="modal-box max-w-md">
        <h2 id="assignment-modal-title" className="text-lg font-semibold">
          {modal.isEditMode ? "Einsatz bearbeiten" : "Einsatz erstellen"}
        </h2>

        {modal.errorMessage ? (
          <p className="mt-3 text-sm text-error">{modal.errorMessage}</p>
        ) : null}

        <section className="mt-4 flex flex-col gap-3">
          <label className="form-control w-full">
            <span className="label-text mb-1">Projekt</span>
            <input
              ref={modal.filterInputRef}
              type="text"
              className="input input-bordered w-full"
              value={modal.filter}
              placeholder="Projekt suchen..."
              onChange={(e) => modal.changeFilter(e.target.value)}
              onKeyDown={modal.handleProjectKeyDown}
              disabled={modal.isSaving}
              role="combobox"
              aria-expanded={modal.displayedProjects.length > 0}
              aria-controls="assignment-project-results"
            />
          </label>
          {modal.selectedProjectRef ? (
            <p className="text-sm">
              Ausgewählt: <strong>{modal.selectedProjectName}</strong>
            </p>
          ) : null}
          {modal.searchError ? (
            <p className="text-sm text-error">{modal.searchError}</p>
          ) : null}
          <ProjectResultList
            projects={modal.displayedProjects}
            highlightedIndex={modal.highlightedIndex}
            onSelect={modal.selectProject}
          />
          <SuggestionEmptyState
            filter={modal.filter}
            suggestionsLoaded={modal.suggestionsLoaded}
            suggestionCount={modal.suggestionCount}
          />
        </section>

        <section className="modal-action">
          {modal.isEditMode ? (
            <button
              type="button"
              className="btn btn-sm btn-error mr-auto"
              onClick={modal.openDeleteConfirm}
              disabled={modal.isSaving}
            >
              Löschen
            </button>
          ) : null}
          <button
            type="button"
            className="btn btn-sm"
            onClick={modal.requestClose}
            disabled={modal.isSaving}
          >
            Abbrechen
          </button>
          <button
            type="button"
            className="btn btn-sm btn-primary"
            onClick={modal.handleSave}
            disabled={
              modal.isSaving || (!modal.isEditMode && !modal.selectedProjectRef)
            }
          >
            {modal.isSaving ? "Speichere..." : "Speichern"}
          </button>
        </section>
      </section>
      <button
        type="button"
        className="modal-backdrop"
        onClick={modal.requestClose}
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
  onSave: (action: ModalSaveAction) => void;
  onClose: () => void;
  showDeleteConfirm?: boolean;
  showUnsavedConfirm?: boolean;
}
