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

const assignmentFallbackColor = "bg-base-300";

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

const categoryChromaCap = 0.14;

/**
 * Re-renders a Daylite category color at the theme's event lightness, keeping its
 * hue and capping its chroma, so arbitrary user-chosen colors stay at an even
 * visual weight across the grid. Returns null for an unparsable value.
 */
export function categoryColorStyle(hexColor: string): string | null {
  const rgb = parseHexColor(hexColor);
  if (!rgb) return null;

  const [red, green, blue] = rgb.map(toLinearChannel);
  const long = Math.cbrt(
    0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue,
  );
  const medium = Math.cbrt(
    0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue,
  );
  const short = Math.cbrt(
    0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue,
  );

  const a = 1.9779984951 * long - 2.428592205 * medium + 0.4505937099 * short;
  const b = 0.0259040371 * long + 0.7827717662 * medium - 0.808675766 * short;
  const chroma = Math.min(Math.hypot(a, b), categoryChromaCap);
  const hue = ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360;

  return `oklch(var(--event-category-l) ${chroma.toFixed(4)} ${hue.toFixed(2)})`;
}

function toLinearChannel(channel: number): number {
  const normalized = channel / 255;
  return normalized <= 0.04045
    ? normalized / 12.92
    : ((normalized + 0.055) / 1.055) ** 2.4;
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
          : assignmentFallbackColor;
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
