import type { CalendarCellEvent } from "../generated/tauri";

/** A resolved event ready for rendering in a timetable cell. */
export interface CellEvent {
  uid: string;
  kind: "assignment" | "bare";
  title: string;
  /** Tailwind CSS background class derived from Daylite project status. */
  color: string;
  /** Start time in HH:MM format. Null for all-day events. */
  startTime: string | null;
  /** End time in HH:MM format. Null for all-day events. */
  endTime: string | null;
}

/** Maps a Daylite project status string to a Tailwind background class. */
function projectStatusToColor(status: string | null | undefined): string {
  switch (status) {
    case "in_progress":
      return "bg-secondary";
    case "done":
      return "bg-success";
    case "abandoned":
      return "bg-neutral";
    case "cancelled":
      return "bg-neutral";
    case "deferred":
      return "bg-warning";
    case "new_status":
      return "bg-primary";
    default:
      return "bg-base-300";
  }
}

/** Converts a `CalendarCellEvent` to a `CellEvent` ready for rendering. */
export function toCellEvent(event: CalendarCellEvent): CellEvent {
  const color =
    event.kind === "bare"
      ? "bg-base-200"
      : projectStatusToColor(event.projectStatus);
  return {
    uid: event.uid,
    kind: event.kind,
    title: event.title,
    color,
    startTime: event.startTime,
    endTime: event.endTime,
  };
}
