import type { CalendarCellEvent } from "../generated/tauri";
import {
  type ProjectCategoryColors,
  projectCategoryColor,
} from "../services/daylite-categories";

export interface CellEvent {
  uid: string;
  kind: "assignment" | "bare" | "absence";
  title: string;
  color: string;
  /** Start time in HH:MM format. Null for all-day events. */
  startTime: string | null;
  /** End time in HH:MM format. Null for all-day events. */
  endTime: string | null;
  href: string | null;
  /** Daylite project reference (e.g. "/v1/projects/42"). Null for non-assignment events. */
  projectRef: string | null;
  /** Daylite project status. Null for non-assignments and for pending or unresolved projects. */
  projectStatus: string | null;
  /** Daylite category color of the project, shown as a card strip. Null without a category color. */
  categoryColor: string | null;
  /** Position among the day's assignments. Null until the event has an order index. */
  orderIndex: number | null;
  /** Project was not cached when the week loaded and is still being resolved. */
  projectPending: boolean;
}

const absenceCodeColors: Record<string, string> = {
  ub: "bg-(--color-absence-vacation)/50",
  su: "bg-(--color-absence-vacation)/30",
  uu: "bg-(--color-absence-vacation)/15",
  kr: "bg-(--color-absence-sick)/40",
  kro: "bg-(--color-absence-sick)/20",
  fa: "bg-(--color-absence-special)/30",
};

const defaultAbsenceColor = "bg-info/30";

const neutralSurfaceColor = "bg-base-200";

function absenceCategoryColor(title: string): string {
  const code = title.trim().split(/\s+/)[0]?.toLowerCase() ?? "";
  return absenceCodeColors[code] ?? defaultAbsenceColor;
}

/** No status while the project is pending or could not be read. */
export function isUnresolvedAssignment(event: CellEvent): boolean {
  return (
    event.projectRef !== null && !event.projectStatus && !event.projectPending
  );
}

/** Highlight key of a card. Null for bare events and absences, which cannot be highlighted. */
export function highlightKey(event: CellEvent): string | null {
  return event.kind === "assignment" ? event.projectRef : null;
}

export function isHighlighted(
  event: CellEvent,
  activeKey: string | null,
): boolean {
  // Bare events and absences key to null; equality alone would group them together.
  return activeKey !== null && highlightKey(event) === activeKey;
}

export function nextHighlight(
  current: string | null,
  key: string | null,
): string | null {
  if (key === null) return current;
  return current === key ? null : key;
}

/** Scoped to the week it was activated in. */
export interface ActiveHighlight {
  key: string;
  weekStart: string;
}

export function activeHighlightForWeek(
  highlight: ActiveHighlight | null,
  weekStart: string,
): string | null {
  if (!highlight || highlight.weekStart !== weekStart) return null;
  return highlight.key;
}

/** The backend expands a multi-day absence into per-day events without a start time. */
export function hasAllDayAbsence(events: CellEvent[]): boolean {
  return events.some(
    (event) => event.kind === "absence" && event.startTime === null,
  );
}

export function hasAbsenceConflict(events: CellEvent[]): boolean {
  return (
    events.some((event) => event.kind === "absence") &&
    events.some((event) => event.kind !== "absence")
  );
}

/** Keeps each day's events in the backend's order. */
export function cellEventsByDay(
  events: CalendarCellEvent[],
  categoryColors: ProjectCategoryColors,
): Partial<Record<string, CellEvent[]>> {
  const byDay: Partial<Record<string, CellEvent[]>> = {};
  for (const event of events) {
    const cellEvent = toCellEvent(event, categoryColors);
    const day = byDay[event.date];
    if (day) day.push(cellEvent);
    else byDay[event.date] = [cellEvent];
  }
  return byDay;
}

export function toCellEvent(
  event: CalendarCellEvent,
  categoryColors: ProjectCategoryColors,
): CellEvent {
  const categoryColor =
    event.kind === "assignment"
      ? projectCategoryColor(categoryColors, event.projectCategory)
      : null;
  const color =
    event.kind === "absence"
      ? absenceCategoryColor(event.title)
      : neutralSurfaceColor;
  return {
    uid: event.uid,
    kind: event.kind,
    title: event.title,
    color,
    categoryColor,
    startTime: event.startTime,
    endTime: event.endTime,
    href: event.href,
    projectRef: event.projectRef,
    projectStatus: event.projectStatus,
    orderIndex: event.orderIndex,
    projectPending: event.projectPending,
  };
}
