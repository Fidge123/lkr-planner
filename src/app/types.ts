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
const assignmentLightness = 0.58;

export interface Oklch {
  lightness: number;
  chroma: number;
  hue: number;
}

/**
 * Background color for an assignment card: the Daylite category color re-rendered
 * at a fixed lightness with its chroma capped, so arbitrary user-chosen colors stay
 * at an even visual weight across the grid. An assignment with no category (or an
 * unparsable one) gets the achromatic color at that same lightness.
 *
 * The result is a plain hex string on purpose. WKWebView, which the desktop app
 * runs on, drops a declaration that resolves a `var()` inside `oklch()`, leaving
 * the card with DaisyUI's default button fill.
 */
export function assignmentBackgroundColor(
  categoryColor: string | null,
): string {
  const source = categoryColor ? oklchFromHex(categoryColor) : null;
  return hexFromOklch({
    lightness: assignmentLightness,
    chroma: source ? Math.min(source.chroma, categoryChromaCap) : 0,
    hue: source?.hue ?? 0,
  });
}

export function oklchFromHex(hexColor: string): Oklch | null {
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

  return {
    lightness:
      0.2104542553 * long + 0.793617785 * medium - 0.0040720468 * short,
    chroma: Math.hypot(a, b),
    hue: ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360,
  };
}

function hexFromOklch({ lightness, chroma, hue }: Oklch): string {
  // Not every hue reaches the chroma cap inside sRGB, so back off until it fits
  // rather than clipping channels, which would drag the hue off target.
  let candidate = chroma;
  while (candidate > 0) {
    const rgb = srgbFromOklch(lightness, candidate, hue);
    if (rgb.every((channel) => channel >= -0.0001 && channel <= 1.0001)) {
      return toHex(rgb);
    }
    candidate -= 0.005;
  }

  return toHex(srgbFromOklch(lightness, 0, hue));
}

function srgbFromOklch(
  lightness: number,
  chroma: number,
  hue: number,
): [number, number, number] {
  const radians = (hue * Math.PI) / 180;
  const a = chroma * Math.cos(radians);
  const b = chroma * Math.sin(radians);

  const long = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const medium = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const short = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;

  return [
    4.0767416621 * long - 3.3077115913 * medium + 0.2309699292 * short,
    -1.2684380046 * long + 2.6097574011 * medium - 0.3413193965 * short,
    -0.0041960863 * long - 0.7034186147 * medium + 1.707614701 * short,
  ];
}

function toHex(linearRgb: [number, number, number]): string {
  const channels = linearRgb.map((channel) => {
    const clamped = Math.min(Math.max(channel, 0), 1);
    const encoded =
      clamped <= 0.0031308
        ? clamped * 12.92
        : 1.055 * clamped ** (1 / 2.4) - 0.055;
    return Math.round(encoded * 255)
      .toString(16)
      .padStart(2, "0");
  });

  return `#${channels.join("")}`;
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
        : "";
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
