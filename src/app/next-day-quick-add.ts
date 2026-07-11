import type { CalendarCellEvent } from "../generated/tauri";

export interface GhostSuggestion {
  date: string;
  projectRef: string;
  projectName: string;
}

export type ModalSaveAction =
  | { kind: "create"; date: string; projectRef: string; projectName: string }
  | { kind: "edit" }
  | { kind: "delete" };

export function nextVisibleDay(
  isoWeekDays: string[],
  date: string,
): string | null {
  const index = isoWeekDays.indexOf(date);
  if (index === -1 || index === isoWeekDays.length - 1) return null;
  return isoWeekDays[index + 1];
}

export function nextGhostState(
  current: GhostSuggestion | null,
  action: ModalSaveAction,
  isoWeekDays: string[],
): GhostSuggestion | null {
  if (action.kind === "delete") return null;
  if (action.kind === "edit") return current;
  const target = nextVisibleDay(isoWeekDays, action.date);
  if (!target) return null;
  return {
    date: target,
    projectRef: action.projectRef,
    projectName: action.projectName,
  };
}

export function isGhostVisible(
  ghost: GhostSuggestion | null,
  date: string,
  eventsOnDate: CalendarCellEvent[],
): boolean {
  return ghost !== null && ghost.date === date && eventsOnDate.length === 0;
}
