import type { CalendarCellEvent } from "../generated/tauri";

/** A translucent next-day quick-add ghost: the project just created, offered on the next visible day. */
export interface GhostSuggestion {
  date: string;
  projectRef: string;
  projectName: string;
}

/** The outcome of a save in the assignment modal. Only "create" ever produces a ghost. */
export type ModalSaveAction =
  | { kind: "create"; date: string; projectRef: string; projectName: string }
  | { kind: "edit" }
  | { kind: "delete" };

/** Returns the ISO date of the next visible day after `date`, or null when `date` is the last visible day. */
export function nextVisibleDay(
  isoWeekDays: string[],
  date: string,
): string | null {
  const index = isoWeekDays.indexOf(date);
  if (index === -1 || index === isoWeekDays.length - 1) return null;
  return isoWeekDays[index + 1];
}

// Centralizes every rule for when a ghost appears or persists: a create sets
// it on the next visible day (or clears it past the last visible day), an
// edit leaves it untouched, a delete always clears it.
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

// A ghost is only shown while its target day still holds no events for the
// employee; re-derived at render time so a reload that fills the day hides it
// automatically, with no separate bookkeeping.
export function isGhostVisible(
  ghost: GhostSuggestion | null,
  date: string,
  eventsOnDate: CalendarCellEvent[],
): boolean {
  return ghost !== null && ghost.date === date && eventsOnDate.length === 0;
}
