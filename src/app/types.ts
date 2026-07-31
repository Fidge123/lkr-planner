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
  /** Hex color of the resolved project's Daylite category. Null when `color` carries the styling instead. */
  categoryColor: string | null;
}

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

const absenceCodeColors: Record<string, string> = {
  ub: "bg-(--color-absence-vacation)/50",
  su: "bg-(--color-absence-vacation)/30",
  uu: "bg-(--color-absence-vacation)/15",
  kr: "bg-(--color-absence-sick)/40",
  kro: "bg-(--color-absence-sick)/20",
  fa: "bg-(--color-absence-special)/30",
};

const defaultAbsenceColor = "bg-info/30";

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

export function readableTextColor(hexColor: string): string {
  const rgb = parseHexColor(hexColor);
  if (!rgb) return "#ffffff";

  const [r, g, b] = rgb.map((channel) => {
    const normalized = channel / 255;
    return normalized <= 0.03928
      ? normalized / 12.92
      : ((normalized + 0.055) / 1.055) ** 2.4;
  });
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

  return luminance > 0.35 ? "#1f2937" : "#ffffff";
}

function parseHexColor(hexColor: string): [number, number, number] | null {
  const hex = hexColor.trim().replace(/^#/, "");
  const expanded =
    hex.length === 3
      ? hex
          .split("")
          .map((char) => char + char)
          .join("")
      : hex;
  if (!/^[0-9a-f]{6}$/i.test(expanded)) return null;

  return [
    Number.parseInt(expanded.slice(0, 2), 16),
    Number.parseInt(expanded.slice(2, 4), 16),
    Number.parseInt(expanded.slice(4, 6), 16),
  ];
}

export function toCellEvent(event: CalendarCellEvent): CellEvent {
  const categoryColor =
    event.kind === "assignment" ? (event.categoryColor ?? null) : null;
  const color =
    event.kind === "absence"
      ? absenceCategoryColor(event.title)
      : event.kind === "bare"
        ? "bg-base-200"
        : categoryColor
          ? ""
          : projectStatusToColor(event.projectStatus);
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
