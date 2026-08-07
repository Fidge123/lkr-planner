import type { DayliteProjectSummary } from "../../generated/tauri";
import type { ModalSaveAction } from "../next-day-quick-add";

const fixedAppointmentCategory = "Termin FIX geplant";

export const fixedAppointmentNotice =
  "Dieser Termin ist als „Termin FIX geplant“ gesperrt und kann nicht bearbeitet oder gelöscht werden.";

// Advisory only: the backend re-derives this per write and is the real enforcement.
export function isProtectedAssignment(
  projectCategory: string | null | undefined,
): boolean {
  return projectCategory === fixedAppointmentCategory;
}

const genericWriteError = "Die Änderung konnte nicht gespeichert werden.";

// Keyed on the status: an error carrying no message must not read as success.
export function commandErrorMessage(result: {
  status: string;
  error?: string;
}): string | null {
  return result.status === "error" ? result.error || genericWriteError : null;
}

export function resolveDisplayedProjects(
  filter: string,
  suggestions: DayliteProjectSummary[],
  results: DayliteProjectSummary[],
): DayliteProjectSummary[] {
  return filter.length === 0 ? suggestions : results;
}

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

export function resolveEscapeAction(filter: string): "clear" | "close" {
  return filter.length > 0 ? "clear" : "close";
}

export type AssignmentWriteIntent = "create" | "update" | "missing-href";

// An edit can only be written back through the assignment's CalDAV resource URL.
// Without one the write must fail visibly: treating it as a create would duplicate the assignment and leave the original untouched.
export function resolveWriteIntent(
  isEditMode: boolean,
  href: string | null | undefined,
): AssignmentWriteIntent {
  if (!isEditMode) return "create";
  return href ? "update" : "missing-href";
}

export function resolveSaveAction(
  isEditMode: boolean,
  date: string,
  projectRef: string,
  projectName: string,
): ModalSaveAction {
  return isEditMode
    ? { kind: "edit" }
    : { kind: "create", date, projectRef, projectName };
}
