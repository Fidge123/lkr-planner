import type { CalendarCellEvent } from "../generated/tauri";

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
  /** Daylite project status. Null for non-assignment events and for assignments whose project could not be resolved. */
  projectStatus: string | null;
  /** Color of the resolved project's Daylite category, shown as a strip on the card. Null when the project has no category color. */
  categoryColor: string | null;
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

export function hasAbsenceConflict(events: CellEvent[]): boolean {
  return (
    events.some((event) => event.kind === "absence") &&
    events.some((event) => event.kind !== "absence")
  );
}

export function toCellEvent(event: CalendarCellEvent): CellEvent {
  const categoryColor =
    event.kind === "assignment" ? (event.categoryColor ?? null) : null;
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
  };
}
