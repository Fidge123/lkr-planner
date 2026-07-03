import { useEffect, useRef, useState } from "react";
import {
  type CalendarCellEvent,
  commands,
  type DayliteProjectSummary,
} from "../../generated/tauri";
import { recordLastAssignedProject } from "../../services/assignment-suggestions";
import { useAssignmentDefaultSuggestions } from "../hooks/use-assignment-default-suggestions";
import { useAssignmentProjectSearch } from "../hooks/use-assignment-project-search";

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
    // Focus the filter so the user can start typing immediately.
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
      result = await commands.updateAssignment(
        assignment.href,
        assignment.uid,
        date,
        selectedProjectRef,
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
    if (selectedProjectRef) {
      recordLastAssignedProject({
        self: selectedProjectRef,
        name: projectName,
      });
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
  onSave: () => void;
  onClose: () => void;
  showDeleteConfirm?: boolean;
  showUnsavedConfirm?: boolean;
}

// Renders the currently displayed result list. Returns nothing when empty, so
// an empty filter keeps the list in its empty default state.
export function ProjectResultList({
  projects,
  highlightedIndex,
  onSelect,
}: ProjectResultListProps) {
  const activeRef = useRef<HTMLButtonElement>(null);

  // Keep the highlighted item visible while navigating with the keyboard.
  useEffect(() => {
    if (highlightedIndex < 0) return;
    activeRef.current?.scrollIntoView({ block: "nearest" });
  }, [highlightedIndex]);

  if (projects.length === 0) return null;

  return (
    <ul
      id="assignment-project-results"
      className="menu menu-sm bg-base-200 rounded-box w-full p-1"
    >
      {projects.map((project, index) => {
        const isActive = index === highlightedIndex;
        return (
          <li key={project.self}>
            <button
              ref={isActive ? activeRef : undefined}
              type="button"
              aria-current={isActive}
              className={
                isActive ? "bg-primary text-primary-content" : undefined
              }
              onClick={() => onSelect(project)}
            >
              {project.name}
            </button>
          </li>
        );
      })}
    </ul>
  );
}

interface ProjectResultListProps {
  projects: DayliteProjectSummary[];
  highlightedIndex: number;
  onSelect: (project: DayliteProjectSummary) => void;
}

// German empty-state message when neither a recent nor overdue projects exist.
// Only shown for the empty filter (the suggestion state) and once loading has
// finished, so it does not flash while the overdue query is in flight.
export function SuggestionEmptyState({
  filter,
  suggestionsLoaded,
  suggestionCount,
}: SuggestionEmptyStateProps) {
  if (filter.length > 0 || !suggestionsLoaded || suggestionCount > 0) {
    return null;
  }
  return <p className="text-sm">Keine Vorschläge verfügbar</p>;
}

interface SuggestionEmptyStateProps {
  filter: string;
  suggestionsLoaded: boolean;
  suggestionCount: number;
}

// An empty filter shows the default suggestions (BL-031); any filter text
// shows the live search results (BL-032). Clearing the filter or resetting it
// via Escape therefore restores the suggestions.
export function resolveDisplayedProjects(
  filter: string,
  suggestions: DayliteProjectSummary[],
  results: DayliteProjectSummary[],
): DayliteProjectSummary[] {
  return filter.length === 0 ? suggestions : results;
}

// Clamps arrow-key movement to the bounds of the displayed list. From the
// unhighlighted state (-1), Arrow Down lands on the first item.
export function nextHighlightIndex(
  current: number,
  length: number,
  direction: 1 | -1,
): number {
  if (length === 0) return -1;
  const next = current + direction;
  if (next < 0) return 0;
  if (next > length - 1) return length - 1;
  return next;
}

// Escape clears a non-empty filter (returning to the empty default state);
// on an empty filter it falls through to the modal close flow.
export function resolveEscapeAction(filter: string): "clear" | "close" {
  return filter.length > 0 ? "clear" : "close";
}
