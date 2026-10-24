import {
  type DayliteProjectSummary,
  FIXED_APPOINTMENT_CATEGORY,
} from "../../generated/tauri";
import type { ModalSaveAction } from "../next-day-quick-add";

export const fixedAppointmentNotice =
  "Dieser Termin ist als „Termin FIX geplant“ gesperrt. Zum Bearbeiten oder Löschen bitte entsperren.";

// Advisory only: the backend re-derives this per write and is the real enforcement.
export function isProtectedAssignment(
  projectCategory: string | null | undefined,
): boolean {
  return projectCategory === FIXED_APPOINTMENT_CATEGORY;
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

// Edits write back only through the CalDAV resource URL.
// Falling back to a create without one would duplicate the assignment.
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
