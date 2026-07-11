import type { DayliteProjectSummary } from "../../generated/tauri";
import type { ModalSaveAction } from "../next-day-quick-add";

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
