import { useCallback, useEffect, useRef, useState } from "react";
import {
  type CalendarCellEvent,
  commands,
  type DayliteProjectSummary,
} from "../../generated/tauri";
import { recordLastAssignedProject } from "../../services/assignment-suggestions";
import {
  commandErrorMessage,
  isProtectedAssignment,
  nextHighlightIndex,
  resolveDisplayedProjects,
  resolveEscapeAction,
  resolveSaveAction,
  resolveWriteIntent,
} from "../components/assignment-modal-logic";
import type { ModalSaveAction } from "../next-day-quick-add";
import { useAssignmentDefaultSuggestions } from "./use-assignment-default-suggestions";
import { useAssignmentProjectSearch } from "./use-assignment-project-search";
import { useProjectCategoryColors } from "./use-project-category-colors";

const missingHrefMessage =
  "Dieser Einsatz kann nicht bearbeitet werden, da er keine Kalender-Adresse hat. Bitte die Ansicht neu laden.";

export function useAssignmentModal({
  isOpen,
  assignment,
  employeeReference,
  date,
  onSave,
  onClose,
  initialShowDeleteConfirm,
  initialShowUnsavedConfirm,
}: Input) {
  const isEditMode = assignment !== null;

  const [filter, setFilter] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const [selectedProjectRef, setSelectedProjectRef] = useState<string>(
    assignment?.projectRef ?? "",
  );
  const [selectedProjectName, setSelectedProjectName] = useState<string>(
    assignment?.title ?? "",
  );
  const [selectedProjectCategory, setSelectedProjectCategory] = useState<
    string | null
  >(null);
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(
    initialShowDeleteConfirm,
  );
  const [showUnsavedConfirm, setShowUnsavedConfirm] = useState(
    initialShowUnsavedConfirm,
  );
  const [isDirty, setIsDirty] = useState(false);
  const filterInputRef = useRef<HTMLInputElement>(null);

  const isProtected =
    isEditMode && isProtectedAssignment(assignment.projectCategory);

  const { results, errorMessage: searchError } =
    useAssignmentProjectSearch(filter);
  const { suggestions, suggestionsLoaded } =
    useAssignmentDefaultSuggestions(isOpen);
  const displayedProjects = resolveDisplayedProjects(
    filter,
    suggestions,
    results,
  );
  const categoryColors = useProjectCategoryColors(isOpen);

  useEffect(() => {
    if (!isOpen) return;
    setErrorMessage(null);
    setIsSaving(false);
    setShowDeleteConfirm(initialShowDeleteConfirm);
    setShowUnsavedConfirm(initialShowUnsavedConfirm);
    setSelectedProjectRef(assignment?.projectRef ?? "");
    setSelectedProjectName(assignment?.title ?? "");
    setSelectedProjectCategory(null);
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

  // A callback ref, not an effect: the dialog element is remounted whenever the modal swaps to the delete or unsaved-changes dialog and back,
  // so an effect keyed on `isOpen` would leave the listener detached after the swap.
  // requestClose is read through a ref because the listener outlives the render that attached it.
  const requestCloseRef = useRef<() => void>(() => {});
  const dialogRef = useCallback((dialog: HTMLDialogElement | null) => {
    if (!dialog) return;
    const handleCancel = (event: Event) => {
      event.preventDefault();
      requestCloseRef.current();
    };
    dialog.addEventListener("cancel", handleCancel);
    return () => dialog.removeEventListener("cancel", handleCancel);
  }, []);

  const requestClose = () => {
    if (isSaving) return;
    if (isDirty) {
      setShowUnsavedConfirm(true);
      return;
    }
    onClose();
  };
  requestCloseRef.current = requestClose;

  const selectProject = (project: DayliteProjectSummary) => {
    setSelectedProjectRef(project.self);
    setSelectedProjectName(project.name);
    setSelectedProjectCategory(project.category ?? null);
    setIsDirty(true);
    setFilter("");
    setHighlightedIndex(-1);
  };

  const changeFilter = (value: string) => {
    setFilter(value);
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
      event.preventDefault();
      changeFilter("");
    }
  };

  const handleSave = async () => {
    if (resolveWriteIntent(isEditMode, assignment?.href) === "missing-href") {
      setErrorMessage(missingHrefMessage);
      return;
    }

    setIsSaving(true);
    setErrorMessage(null);

    const projectName = selectedProjectName || assignment?.title || "";

    const result: { status: string; error?: string } = assignment?.href
      ? await commands.updateAssignment({
          href: assignment.href,
          uid: assignment.uid,
          date,
          projectRef: selectedProjectRef,
          projectName,
          // Editing an assignment must not move it within its day.
          orderIndex: null,
        })
      : await commands.createAssignment({
          employeeReference,
          date,
          projectRef: selectedProjectRef,
          projectName,
        });

    const writeError = commandErrorMessage(result);
    if (writeError) {
      setErrorMessage(writeError);
      setIsSaving(false);
      return;
    }
    if (selectedProjectRef) {
      recordLastAssignedProject({
        self: selectedProjectRef,
        name: projectName,
        category: selectedProjectCategory,
      });
    }
    onSave(
      resolveSaveAction(isEditMode, date, selectedProjectRef, projectName),
    );
  };

  const handleDelete = async () => {
    if (!assignment?.href) {
      setErrorMessage(missingHrefMessage);
      return;
    }
    setIsSaving(true);
    setErrorMessage(null);
    const result = await commands.deleteAssignment(assignment.href);
    const deleteError = commandErrorMessage(result);
    if (deleteError) {
      setErrorMessage(deleteError);
      setIsSaving(false);
      return;
    }
    onSave({ kind: "delete" });
  };

  return {
    isEditMode,
    isProtected,
    dialogRef,
    filterInputRef,
    filter,
    highlightedIndex,
    displayedProjects,
    categoryColors,
    selectedProjectRef,
    selectedProjectName,
    isSaving,
    errorMessage,
    searchError,
    suggestionsLoaded,
    suggestionCount: suggestions.length,
    showSuggestionPlaceholder: filter.length === 0 && !suggestionsLoaded,
    showDeleteConfirm,
    showUnsavedConfirm,
    requestClose,
    selectProject,
    changeFilter,
    handleProjectKeyDown,
    handleSave,
    handleDelete,
    openDeleteConfirm: () => setShowDeleteConfirm(true),
    cancelDeleteConfirm: () => setShowDeleteConfirm(false),
    continueEditing: () => setShowUnsavedConfirm(false),
  };
}

interface Input {
  isOpen: boolean;
  assignment: CalendarCellEvent | null;
  employeeReference: string;
  date: string;
  onSave: (action: ModalSaveAction) => void;
  onClose: () => void;
  initialShowDeleteConfirm: boolean;
  initialShowUnsavedConfirm: boolean;
}
