import { useEffect, useRef, useState } from "react";
import {
  type CalendarCellEvent,
  commands,
  type DayliteProjectSummary,
} from "../../generated/tauri";
import { recordLastAssignedProject } from "../../services/assignment-suggestions";
import { useAssignmentDefaultSuggestions } from "../hooks/use-assignment-default-suggestions";
import { useAssignmentProjectSearch } from "../hooks/use-assignment-project-search";
import type { ModalSaveAction } from "../next-day-quick-add";
import {
  nextHighlightIndex,
  resolveDisplayedProjects,
  resolveEscapeAction,
  resolveSaveAction,
} from "./assignment-modal-logic";
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
  const isEditMode = assignment !== null;

  const [filter, setFilter] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const [selectedProjectRef, setSelectedProjectRef] = useState<string>(
    assignment?.projectRef ?? "",
  );
  const [selectedProjectName, setSelectedProjectName] = useState<string>(
    assignment?.title ?? "",
  );
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
  const filterInputRef = useRef<HTMLInputElement>(null);

  const { results, errorMessage: searchError } =
    useAssignmentProjectSearch(filter);
  const { suggestions, suggestionsLoaded } =
    useAssignmentDefaultSuggestions(isOpen);
  const displayedProjects = resolveDisplayedProjects(
    filter,
    suggestions,
    results,
  );

  useEffect(() => {
    if (!isOpen) return;
    setErrorMessage(null);
    setIsSaving(false);
    setShowDeleteConfirm(initialShowDeleteConfirm);
    setShowUnsavedConfirm(initialShowUnsavedConfirm);
    setSelectedProjectRef(assignment?.projectRef ?? "");
    setSelectedProjectName(assignment?.title ?? "");
    setFilter("");
    setHighlightedIndex(-1);
    setIsDirty(false);
    filterInputRef.current?.focus();
  }, [
    isOpen,
    initialShowDeleteConfirm,
    initialShowUnsavedConfirm,
    assignment?.projectRef,
    assignment?.title,
  ]);

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
      <UnsavedChangesDialog
        onContinueEditing={() => setShowUnsavedConfirm(false)}
        onDiscard={onClose}
      />
    );
  }

  if (showDeleteConfirm) {
    return (
      <DeleteConfirmDialog
        isDeleting={isSaving}
        errorMessage={errorMessage}
        onCancel={() => setShowDeleteConfirm(false)}
        onConfirm={async () => {
          if (!assignment?.href) return;
          setIsSaving(true);
          setErrorMessage(null);
          const result = await commands.deleteAssignment(assignment.href);
          if (result.status === "error") {
            setErrorMessage(result.error);
            setIsSaving(false);
            return;
          }
          onSave({ kind: "delete" });
        }}
        onRequestClose={requestClose}
      />
    );
  }

  const selectProject = (project: DayliteProjectSummary) => {
    setSelectedProjectRef(project.self);
    setSelectedProjectName(project.name);
    setIsDirty(true);
    // Selecting returns the list to its empty default state.
    setFilter("");
    setHighlightedIndex(-1);
  };

  const handleProjectKeyDown = (
    event: React.KeyboardEvent<HTMLInputElement>,
  ) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setHighlightedIndex((index) =>
        nextHighlightIndex(index, displayedProjects.length, 1),
      );
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setHighlightedIndex((index) =>
        nextHighlightIndex(index, displayedProjects.length, -1),
      );
      return;
    }
    if (event.key === "Enter") {
      const highlighted = displayedProjects[highlightedIndex];
      if (highlighted) {
        event.preventDefault();
        selectProject(highlighted);
      }
      return;
    }
    if (event.key === "Escape" && resolveEscapeAction(filter) === "clear") {
      // Intercept before the native <dialog> cancel: clear instead of close.
      event.preventDefault();
      setFilter("");
      setHighlightedIndex(-1);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setErrorMessage(null);

    const projectName = selectedProjectName || assignment?.title || "";

    let result: { status: string; error?: string };
    if (isEditMode && assignment.href) {
      result = await commands.updateAssignment({
        href: assignment.href,
        uid: assignment.uid,
        date,
        projectRef: selectedProjectRef,
        projectName,
      });
    } else {
      result = await commands.createAssignment({
        employeeReference,
        date,
        projectRef: selectedProjectRef,
        projectName,
      });
    }

    if (result.status === "error") {
      setErrorMessage((result as { status: "error"; error: string }).error);
      setIsSaving(false);
      return;
    }
    if (selectedProjectRef) {
      recordLastAssignedProject({
        self: selectedProjectRef,
        name: projectName,
      });
    }
    onSave(
      resolveSaveAction(isEditMode, date, selectedProjectRef, projectName),
    );
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
          <label className="form-control w-full">
            <span className="label-text mb-1">Projekt</span>
            <input
              ref={filterInputRef}
              type="text"
              className="input input-bordered w-full"
              value={filter}
              placeholder="Projekt suchen..."
              onChange={(e) => {
                // Typing changes the result set, so the previous highlight is stale.
                setFilter(e.target.value);
                setHighlightedIndex(-1);
              }}
              onKeyDown={handleProjectKeyDown}
              disabled={isSaving}
              role="combobox"
              aria-expanded={displayedProjects.length > 0}
              aria-controls="assignment-project-results"
            />
          </label>
          {selectedProjectRef ? (
            <p className="text-sm">
              Ausgewählt: <strong>{selectedProjectName}</strong>
            </p>
          ) : null}
          {searchError ? (
            <p className="text-sm text-error">{searchError}</p>
          ) : null}
          <ProjectResultList
            projects={displayedProjects}
            highlightedIndex={highlightedIndex}
            onSelect={selectProject}
          />
          <SuggestionEmptyState
            filter={filter}
            suggestionsLoaded={suggestionsLoaded}
            suggestionCount={suggestions.length}
          />
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
  onSave: (action: ModalSaveAction) => void;
  onClose: () => void;
  showDeleteConfirm?: boolean;
  showUnsavedConfirm?: boolean;
}
