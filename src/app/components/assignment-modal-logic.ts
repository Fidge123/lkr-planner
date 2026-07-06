import type { DayliteProjectSummary } from "../../generated/tauri";
import type { ModalSaveAction } from "../next-day-quick-add";

// An empty filter shows the default suggestions; any filter text shows the live
// search results. Clearing the filter (or resetting it via Escape) therefore
// restores the suggestions.
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

// Only a create carries enough information (and intent) to seed a next-day
// ghost; an edit never should, no matter what project ended up selected.
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
